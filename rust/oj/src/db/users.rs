//! Database operations on the `users` table.

use super::connection::ConnectionPool;
use super::schema::users::dsl;
use crate::error::*;
use diesel::prelude::*;
use serde::Serialize;

/// Checks that the given name is either not used or is used by the user of the given id.
fn name_not_used(name: &str, id: Option<i32>, conn: &mut SqliteConnection) -> ApiResult<()> {
    let current_id = dsl::users
        .select(dsl::id)
        .filter(dsl::name.eq(name))
        .first(conn)
        .optional()?;
    if current_id.is_some() && current_id != id {
        Err(ApiError::new(
            ApiErrorType::InvalidArgument,
            format!("User name '{}' already exists.", name),
        ))
    } else {
        Ok(())
    }
}

#[derive(Serialize, Insertable, Queryable)]
#[diesel(table_name = super::schema::users)]
pub struct User {
    pub id: i32,
    name: String,
}

/// Get the list of all users sorted by id.
pub fn get_users(pool: &ConnectionPool) -> ApiResult<Vec<User>> {
    Ok(dsl::users.order(dsl::id).load(&mut pool.get()?)?)
}

/// Change the name of an existing user, errors on duplicated username.
pub fn set_name(id: i32, name: String, pool: &ConnectionPool) -> ApiResult<User> {
    pool.get()?.immediate_transaction(|conn| {
        name_not_used(&name, Some(id), conn)?;
        let change_count = diesel::update(dsl::users)
            .filter(dsl::id.eq(id))
            .set(dsl::name.eq(&name))
            .execute(conn)?;
        if change_count == 0 {
            Err(ApiError::not_found(&format!("User {}", id)))
        } else {
            Ok(User { id, name })
        }
    })
}

/// Get the number of existing users.
pub(super) fn user_count(conn: &mut SqliteConnection) -> ApiResult<i32> {
    Ok(dsl::users.count().get_result::<i64>(conn)? as i32)
}

/// Add a new user, errors on duplicated username.
pub fn add_user(name: String, pool: &ConnectionPool) -> ApiResult<User> {
    pool.get()?.immediate_transaction(|conn| {
        name_not_used(&name, None, conn)?;
        let id = user_count(conn)?;
        let user = User { id, name };
        diesel::insert_into(dsl::users)
            .values(&user)
            .execute(conn)?;
        Ok(user)
    })
}

/// Get the id of the user with the given name.
pub(super) fn get_user_id(name: &str, conn: &mut SqliteConnection) -> ApiResult<Option<i32>> {
    Ok(dsl::users
        .select(dsl::id)
        .filter(dsl::name.eq(name))
        .first(conn)
        .optional()?)
}

pub fn get_single_user(id: i32, pool: &ConnectionPool) -> ApiResult<User> {
    let user = dsl::users
        .filter(dsl::id.eq(id))
        .first(&mut pool.get()?)
        .optional()?;

    match user {
        Some(user) => Ok(user),
        None => Err(ApiError::not_found(&format!("User {}", id))),
    }
}
