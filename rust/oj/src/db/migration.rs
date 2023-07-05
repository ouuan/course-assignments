//! Run/revert database migrations.

use super::connection::ConnectionPool;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub fn initialize_database(delete_data: bool, pool: &ConnectionPool) {
    let mut conn = pool.get().expect("failed to connect to database");
    if delete_data {
        conn.revert_all_migrations(MIGRATIONS)
            .expect("failed to revert migrations");
    }
    conn.run_pending_migrations(MIGRATIONS)
        .expect("failed to run migrations");
}
