pub mod cli;
pub mod config;
pub mod db;
pub mod error;
pub mod judger;
pub mod routes;

/// The time format used in APIs.
const TIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.3fZ";
