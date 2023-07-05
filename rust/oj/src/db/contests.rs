//! Database operations on the `contests` table.

use super::connection::ConnectionPool;
use super::schema::contests::dsl;
use super::users::User;
use super::{contest_problems, contest_users, users};
use crate::error::*;
use crate::TIME_FORMAT;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// The API response representing a contest without the id of the contest.
#[derive(Deserialize, Serialize, Clone)]
pub struct ContestWithoutId {
    name: String,
    // The Deserialize of NaiveDateTime uses a different format, so manually parse instead
    from: String,
    to: String,
    pub problem_ids: Vec<i32>,
    pub user_ids: Vec<i32>,
    submission_limit: i32,
}

/// The API response representing a contest.
#[derive(Serialize)]
pub struct Contest {
    id: i32,
    #[serde(flatten)]
    contest: ContestWithoutId,
}

/// The metadata of the contest without problems and users in the contest.
#[derive(Insertable, Queryable, AsChangeset)]
#[diesel(table_name = super::schema::contests)]
pub(super) struct ContestInfo {
    id: i32,
    name: String,
    pub from: NaiveDateTime,
    pub to: NaiveDateTime,
    pub submission_limit: i32,
}

impl Contest {
    fn new(info: ContestInfo, problem_ids: Vec<i32>, user_ids: Vec<i32>) -> Self {
        Self {
            id: info.id,
            contest: ContestWithoutId {
                name: info.name,
                from: info.from.format(TIME_FORMAT).to_string(),
                to: info.to.format(TIME_FORMAT).to_string(),
                problem_ids,
                user_ids,
                submission_limit: info.submission_limit,
            },
        }
    }
}

/// Validate a `ContestWithoutId` and returns a `ContestInfo` with id = 0 if valid.
/// It doesn't check duplicated users/problems or existence of problems.
fn validate_contest(
    contest: &ContestWithoutId,
    conn: &mut SqliteConnection,
) -> ApiResult<ContestInfo> {
    let user_count = users::user_count(conn)?;
    match contest
        .user_ids
        .iter()
        .find(|&&id| id < 0 || id >= user_count)
    {
        Some(id) => return Err(ApiError::not_found(&format!("User {}", id))),
        None => {}
    };
    Ok(ContestInfo {
        id: 0,
        name: contest.name.clone(),
        from: super::utils::parse_time(&contest.from, "from")?,
        to: super::utils::parse_time(&contest.to, "to")?,
        submission_limit: contest.submission_limit,
    })
}

/// Get the number of contests.
fn contest_count(conn: &mut SqliteConnection) -> ApiResult<i64> {
    Ok(dsl::contests.count().get_result(conn)?)
}

/// Add a new contest.
/// It validates the contest but doesn't check duplicated users/problems or existence of problems.
/// Returns `contest_id` on success.
pub fn add_contest(contest: &ContestWithoutId, pool: &ConnectionPool) -> ApiResult<i32> {
    pool.get()?.immediate_transaction(|conn| {
        let mut contest_info = validate_contest(contest, conn)?;
        let id = contest_count(conn)? as i32 + 1;
        contest_info.id = id;
        diesel::insert_into(dsl::contests)
            .values(contest_info)
            .execute(conn)?;
        contest_users::insert_contest_users(id, &contest.user_ids, conn)?;
        contest_problems::insert_contest_problems(id, &contest.problem_ids, conn)?;
        Ok(id)
    })
}

/// Update an existing contest.
/// It validates the contest but doesn't check duplicated users/problems or existence of problems.
pub fn update_contest(id: i32, contest: &ContestWithoutId, pool: &ConnectionPool) -> ApiResult<()> {
    pool.get()?.immediate_transaction(|conn| {
        get_contest_info(id, conn)?; // check contest existence first
        let mut contest_info = validate_contest(contest, conn)?;
        contest_info.id = id;
        diesel::update(dsl::contests)
            .filter(dsl::id.eq(id))
            .set(&contest_info)
            .execute(conn)?;
        contest_users::delete_contest_users(id, conn)?;
        contest_users::insert_contest_users(id, &contest.user_ids, conn)?;
        contest_problems::delete_contest_problems(id, conn)?;
        contest_problems::insert_contest_problems(id, &contest.problem_ids, conn)?;
        Ok(())
    })
}

/// Get a list of all contests.
pub fn get_all_contests(pool: &ConnectionPool) -> ApiResult<Vec<Contest>> {
    pool.get()?.immediate_transaction(|conn| {
        let contests_info = dsl::contests.order(dsl::id).load::<ContestInfo>(conn)?;
        let mut contests = Vec::new();
        for info in contests_info {
            let user_ids = contest_users::get_contest_users(info.id, conn)?;
            let problem_ids = contest_problems::get_contest_problems(info.id, conn)?;
            contests.push(Contest::new(info, problem_ids, user_ids));
        }
        Ok(contests)
    })
}

/// Get the contest info of a single contest. Returns not-found error if contest not found.
pub(super) fn get_contest_info(id: i32, conn: &mut SqliteConnection) -> ApiResult<ContestInfo> {
    match dsl::contests
        .filter(dsl::id.eq(id))
        .first::<ContestInfo>(conn)
        .optional()?
    {
        Some(info) => Ok(info),
        None => Err(ApiError::not_found(&format!("Contest {}", id))),
    }
}

/// Get the contest info, users and problems of a single contest.
pub fn get_contest(id: i32, pool: &ConnectionPool) -> ApiResult<Contest> {
    pool.get()?.immediate_transaction(|conn| {
        let info = get_contest_info(id, conn)?;
        let user_ids = contest_users::get_contest_users(id, conn)?;
        let problem_ids = contest_problems::get_contest_problems(id, conn)?;
        Ok(Contest::new(info, problem_ids, user_ids))
    })
}

/// Get the users and problems of a single contest.
pub fn get_contest_users_and_problem_ids(
    contest_id: i32,
    pool: &ConnectionPool,
) -> ApiResult<(Vec<User>, Vec<i32>)> {
    pool.get()?.immediate_transaction(|conn| {
        get_contest_info(contest_id, conn)?;
        let users = contest_users::get_contest_users_with_names(contest_id, conn)?;
        let problem_ids = contest_problems::get_contest_problems(contest_id, conn)?;
        Ok((users, problem_ids))
    })
}
