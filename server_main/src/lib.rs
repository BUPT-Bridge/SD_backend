mod router;

use axum::Router;
use db_manager::migrator::Migrator;
use db_manager::*;
use dotenvy::dotenv;
use router::user;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sea_orm_migration::prelude::*;
use std::sync::Arc;
#[allow(unused_imports)]
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub database: Arc<DatabaseConnection>,
}

async fn build_database_connection() -> DatabaseConnection {
    let db_config = DatabaseConfig::from_env().uri();
    let mut options = ConnectOptions::new(&db_config);
    options
        .max_connections(10)
        .min_connections(1)
        .sqlx_logging(false)
        .sqlx_logging_level(log::LevelFilter::Warn);

    Database::connect(options)
        .await
        .expect("failed to connect to database")
}

/// Build the application router under `/api` and attach shared state.
/// Downstream routers should be nested under `/api`.
pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    let database = build_database_connection().await;

    Migrator::refresh(&database).await?;

    let state = AppState {
        database: Arc::new(database),
    };

    let api_router = Router::new()
        .nest("/user", user::register_router())
        .nest("/user", user::login_router())
        .nest("/user", user::modify_router());

    let app = Router::new()
        .nest("/api", api_router)
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await?;

    Ok(())
}
