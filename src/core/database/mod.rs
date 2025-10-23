pub mod operations;
pub mod schema;
use crate::core::utils::db_url;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use tracing::info;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn run_migrations() {
    let database_url = db_url();
    let mut conn = SqliteConnection::establish(&database_url).expect("Error conectando a la DB");

    conn.run_pending_migrations(MIGRATIONS)
        .expect("Error aplicando migraciones");
    info!("DB lista y migraciones aplicadas");
}

pub use operations::{get_db_manager, init_db_manager};
