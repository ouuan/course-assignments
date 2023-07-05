//! Database operations on the `contest_users` table.

use super::schema::contest_users::dsl;
use super::users::User;
use crate::error::ApiResult;
use diesel::prelude::*;

/// Add `user_ids` into the given contest.
pub fn insert_contest_users(
    contest_id: i32,
    user_ids: &[i32],
    conn: &mut SqliteConnection,
) -> ApiResult<()> {
    diesel::insert_into(dsl::contest_users)
        .values(
            user_ids
                .iter()
                .map(|id| (dsl::contest_id.eq(contest_id), dsl::user_id.eq(id)))
                .collect::<Vec<_>>(),
        )
        .execute(conn)?;
    Ok(())
}

/// Delete all users from the given contest.
pub fn delete_contest_users(contest_id: i32, conn: &mut SqliteConnection) -> ApiResult<()> {
    diesel::delete(dsl::contest_users)
        .filter(dsl::contest_id.eq(contest_id))
        .execute(conn)?;
    Ok(())
}

/// Get a list of all user ids in the given contest.
pub fn get_contest_users(contest_id: i32, conn: &mut SqliteConnection) -> ApiResult<Vec<i32>> {
    Ok(dsl::contest_users
        .select(dsl::user_id)
        .filter(dsl::contest_id.eq(contest_id))
        .order(dsl::rowid)
        .load(conn)?)
}

/// Returns whether the given user is in the given contest.
pub fn is_user_in_contest(
    contest_id: i32,
    user_id: i32,
    conn: &mut SqliteConnection,
) -> ApiResult<bool> {
    let count = dsl::contest_users
        .count()
        .filter(dsl::contest_id.eq(contest_id))
        .filter(dsl::user_id.eq(user_id))
        .get_result::<i64>(conn)?;
    Ok(count > 0)
}

/// Get a list of all users in the given contest along with their names.
pub fn get_contest_users_with_names(
    contest_id: i32,
    conn: &mut SqliteConnection,
) -> ApiResult<Vec<User>> {
    use super::schema::users;
    Ok(dsl::contest_users
        .inner_join(users::table)
        .select((dsl::user_id, users::columns::name))
        .filter(dsl::contest_id.eq(contest_id))
        .order(dsl::rowid)
        .load(conn)?)
}
