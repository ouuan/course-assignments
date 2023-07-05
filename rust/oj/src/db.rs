//! Database-related modules.

pub mod connection;
pub mod migration;

pub(crate) mod enums;

pub(crate) mod case_results;
pub(crate) mod contests;
pub(crate) mod jobs;
pub(crate) mod users;

mod contest_problems;
mod contest_users;

mod schema;

mod utils;

lazy_static::lazy_static! {
    /// Path to the database file.
    static ref DATABASE_URL: String = std::env::var("DATABASE_URL").expect("env var DATABASE_URL must be set");
}
