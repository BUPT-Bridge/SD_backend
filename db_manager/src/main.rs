use db_manager;
use db_manager::migrator::Migrator;
use dotenvy::dotenv;
use log;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sea_orm_migration::prelude::*;
use tracing;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_test_writer()
        .init();
    let db_config = db_manager::config::DatabaseConfig::from_env().uri();
    let mut db_connection_options = ConnectOptions::new(&db_config);
    db_connection_options
        .max_connections(10)
        .min_connections(1)
        .sqlx_logging(false)
        .sqlx_logging_level(log::LevelFilter::Info);
    let db: DatabaseConnection = Database::connect(db_connection_options).await?;

    let schema_manager = SchemaManager::new(&db);
    Migrator::refresh(&db).await?;
    assert!(schema_manager.has_table("bakery").await?);
    assert!(schema_manager.has_table("chef").await?);
    Ok(())
}
