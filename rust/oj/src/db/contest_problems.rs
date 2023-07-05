//! Database operations on the `contest_problems` table.

use super::schema::contest_problems::dsl;
use crate::error::ApiResult;
use diesel::prelude::*;

/// Add `problem_ids` into the given contest.
pub fn insert_contest_problems(
    contest_id: i32,
    problem_ids: &[i32],
    conn: &mut SqliteConnection,
) -> ApiResult<()> {
    diesel::insert_into(dsl::contest_problems)
        .values(
            problem_ids
                .iter()
                .map(|id| (dsl::contest_id.eq(contest_id), dsl::problem_id.eq(id)))
                .collect::<Vec<_>>(),
        )
        .execute(conn)?;
    Ok(())
}

/// Delete all problems from the given contest.
pub fn delete_contest_problems(contest_id: i32, conn: &mut SqliteConnection) -> ApiResult<()> {
    diesel::delete(dsl::contest_problems)
        .filter(dsl::contest_id.eq(contest_id))
        .execute(conn)?;
    Ok(())
}

/// Get a list of all problem ids in the given contest.
pub fn get_contest_problems(contest_id: i32, conn: &mut SqliteConnection) -> ApiResult<Vec<i32>> {
    Ok(dsl::contest_problems
        .select(dsl::problem_id)
        .filter(dsl::contest_id.eq(contest_id))
        .order(dsl::rowid)
        .load(conn)?)
}

/// Returns whether the given problem is in the given contest.
pub fn is_problem_in_contest(
    contest_id: i32,
    problem_id: i32,
    conn: &mut SqliteConnection,
) -> ApiResult<bool> {
    let count = dsl::contest_problems
        .count()
        .filter(dsl::contest_id.eq(contest_id))
        .filter(dsl::problem_id.eq(problem_id))
        .get_result::<i64>(conn)?;
    Ok(count > 0)
}
