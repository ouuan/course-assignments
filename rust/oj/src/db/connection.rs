//! Obtain connections to the database.

use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, CustomizeConnection, Pool};
use r2d2::NopErrorHandler;

/// To fix the database locked error. Based on <https://stackoverflow.com/a/57717533>.
#[derive(Debug)]
struct CustomConnection;
impl CustomizeConnection<SqliteConnection, diesel::r2d2::Error> for CustomConnection {
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
        // Remember to use immediate_transaction instead of transaction!
        // See <https://github.com/the-lean-crate/criner/issues/1>
        conn.batch_execute(
            r#"
                PRAGMA foreign_keys = ON;
                PRAGMA journal_mode = WAL;
                PRAGMA synchronous = NORMAL;
                PRAGMA wal_autocheckpoint = 1000;
                PRAGMA wal_checkpoint(TRUNCATE);
                PRAGMA busy_timeout = 10000;
            "#,
        )
        .map_err(diesel::r2d2::Error::QueryError)
    }
}

/// Type alias of a Sqlite connection pool.
pub(crate) type ConnectionPool = Pool<ConnectionManager<SqliteConnection>>;

/// Construct a connection pool to the database.
pub fn connection_pool() -> ConnectionPool {
    let manager = ConnectionManager::<SqliteConnection>::new(super::DATABASE_URL.as_str());
    Pool::builder()
        .max_size(10 + num_cpus::get() as u32)
        // Silent the false alarm "database is locked" at startup
        .error_handler(Box::new(NopErrorHandler))
        .connection_customizer(Box::new(CustomConnection))
        // Use the unchecked version to immediately start the server.
        .build_unchecked(manager)
}
