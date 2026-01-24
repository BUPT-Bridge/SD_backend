use db_manager::{config::DatabaseConfig, migrator::Migrator};
use dotenvy::dotenv;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sea_orm_migration::prelude::*;
use std::sync::Once;

use tracing::Level;

static TRACING: Once = Once::new();

fn init_tracing() {
    TRACING.call_once(|| {
        let _ = tracing_subscriber::fmt()
            .with_max_level(Level::DEBUG)
            .with_test_writer()
            .try_init();
    });
}

async fn setup_connection() -> DatabaseConnection {
    dotenv().ok();
    init_tracing();

    let db_config = DatabaseConfig::from_env().uri();
    let mut options = ConnectOptions::new(&db_config);
    options
        .max_connections(10)
        .min_connections(1)
        .sqlx_logging(false)
        .sqlx_logging_level(log::LevelFilter::Debug);

    Database::connect(options)
        .await
        .expect("failed to connect to database")
}

#[tokio::test]
async fn migration_refresh_creates_expected_tables() -> Result<(), Box<dyn std::error::Error>> {
    let db = setup_connection().await;

    // Drop and recreate all managed tables.
    Migrator::refresh(&db).await?;

    // Verify a few representative tables exist after refresh.
    let schema_manager = SchemaManager::new(&db);
    let expected_tables = ["user", "notice", "policy_file"];

    for table in expected_tables {
        assert!(
            schema_manager.has_table(table).await?,
            "expected table `{}` to exist after migration refresh",
            table
        );
    }

    Ok(())
}
