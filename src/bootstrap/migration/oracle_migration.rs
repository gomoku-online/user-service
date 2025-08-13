use diesel::Connection;
use diesel::migration::MigrationConnection;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::row::NamedRow;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use diesel_oci::{OciConnection, Oracle};
use tracing::info;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/oracle");

pub async fn run_oracle_migrations(conn: &mut impl MigrationHarness<Oracle>) {
    info!("Oracle migration starts");

    conn.run_pending_migrations(MIGRATIONS)
        .expect("failed to run migration");

    info!("Database migration ends");
}
