pub mod entities;
pub mod migrate;

use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::migrate::Migrator;

use crate::config::DatabaseConfig;

pub async fn init_pool(config: &DatabaseConfig) -> PgPool {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.connection_string())
        .await
        .expect("Failed to create database pool");

    pool
}

pub async fn run_migrations(pool: &PgPool) {
    let migrator = Migrator::new(std::env!("CARGO_MANIFEST_DIR"))
        .await
        .expect("Failed to create migrator");

    migrator.run(pool).await.expect("Failed to run migrations");
}
