//! Database operations on the `case_results` table.

use super::connection::ConnectionPool;
use super::enums::JobResult;
use super::jobs;
use super::schema::case_results::dsl;
use crate::error::*;
use diesel::prelude::*;
use serde::Serialize;

/// All columns of the `case_results` table including the the `job_id` field.
#[derive(Insertable, Queryable, AsChangeset)]
#[diesel(table_name = super::schema::case_results)]
struct FullCase {
    job_id: i32,
    id: i32,
    result: JobResult,
    time: i64,
    memory: i64,
    info: String,
}

/// The case that can be used in the API.
#[derive(Serialize, Queryable)]
pub struct Case {
    id: i32,
    result: JobResult,
    time: i64,
    memory: i64,
    info: String,
}

/// Insert cases for a new job.
pub fn init_cases(
    job_id: i32,
    case_count: usize,
    conn: &mut SqliteConnection,
) -> ApiResult<Vec<Case>> {
    let cases = (0..=case_count)
        .map(|id| FullCase {
            job_id,
            id: id as i32,
            result: JobResult::Waiting,
            time: 0,
            memory: 0,
            info: String::new(),
        })
        .collect::<Vec<_>>();
    diesel::insert_into(dsl::case_results)
        .values(&cases)
        .execute(conn)?;
    Ok(cases
        .into_iter()
        .map(|case| Case {
            id: case.id,
            result: case.result,
            time: case.time,
            memory: case.memory,
            info: case.info,
        })
        .collect())
}

/// Update cases for rejudging a job.
pub fn reinit_cases(job_id: i32, conn: &mut SqliteConnection) -> ApiResult<()> {
    diesel::update(dsl::case_results)
        .filter(dsl::job_id.eq(job_id))
        .set((
            dsl::result.eq(JobResult::Waiting),
            dsl::time.eq(0),
            dsl::memory.eq(0),
            dsl::info.eq(""),
        ))
        .execute(conn)?;
    Ok(())
}

/// Get cases of a single job.
pub(super) fn get_cases(job_id: i32, conn: &mut SqliteConnection) -> ApiResult<Vec<Case>> {
    Ok(dsl::case_results
        .select((dsl::id, dsl::result, dsl::time, dsl::memory, dsl::info))
        .filter(dsl::job_id.eq(job_id))
        .order(dsl::id)
        .load(conn)?)
}

/// Data used to update the judge result of a single case.
#[derive(AsChangeset)]
#[diesel(table_name = super::schema::case_results)]
pub struct CaseUpdate {
    pub result: JobResult,
    pub time: i64,
    pub info: String,
}

/// Update a single case and the total score of a job.
///
/// Note: Sometimes the score does not need to be updated. But the `updated_time` of the job always
/// needs to be updated, so it does not cost an extra query to update the score.
pub fn update_case(
    job_id: i32,
    case_id: i32,
    update: &CaseUpdate,
    total_score: f64,
    pool: &ConnectionPool,
) -> ApiResult<()> {
    pool.get()?.immediate_transaction(|conn| {
        diesel::update(dsl::case_results)
            .filter(dsl::job_id.eq(job_id))
            .filter(dsl::id.eq(case_id))
            .set(update)
            .execute(conn)?;
        jobs::update_score(job_id, total_score, conn)?;
        Ok(())
    })
}

/// Get the time of each case except the first one which is compilation.
pub fn get_cases_time(job_id: i32, conn: &mut SqliteConnection) -> ApiResult<Vec<i64>> {
    Ok(dsl::case_results
        .select(dsl::time)
        .filter(dsl::job_id.eq(job_id))
        .filter(dsl::id.gt(0))
        .order(dsl::id)
        .load(conn)?)
}
