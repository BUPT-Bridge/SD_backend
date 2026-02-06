mod router;

use axum::Router;
use db_manager::migrator::Migrator;
use db_manager::*;
use dotenvy::dotenv;
use router::ai_chat;
use router::community_service;
use router::detail_meal;
use router::dinner_provider;
use router::feedback;
use router::health_guide_content;
use router::health_guide_type;
use router::medical_service;
use router::mutil_media;
use router::notice;
use router::policy_file;
use router::policy_type;
use router::resource_service;
use router::service_map_content;
use router::service_map_type;
use router::slide_show;
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
        .nest("/user", user::modify_router())
        .nest("/user", user::info_router())
        .nest("/user", user::apply_permission_router())
        .nest("/user", user::admin_manager_router())
        .nest("/ai_chat", ai_chat::ai_chat_router())
        .nest("/notice", notice::notice_router())
        .nest("/mutil_media", mutil_media::mutil_media_router())
        .nest("/slide_show", slide_show::slide_show_router())
        .nest(
            "/community_service",
            community_service::community_service_router(),
        )
        .nest(
            "/dinner_provider",
            dinner_provider::dinner_provider_router(),
        )
        .nest("/detail_meal", detail_meal::detail_meal_router())
        .nest(
            "/resource_service",
            resource_service::resource_service_router(),
        )
        .nest(
            "/medical_service",
            medical_service::medical_service_router(),
        )
        .nest("/feedback", feedback::feedback_router())
        .nest(
            "/service_map_type",
            service_map_type::service_map_type_router(),
        )
        .nest(
            "/health_guide_type",
            health_guide_type::health_guide_type_router(),
        )
        .nest(
            "/health_guide_content",
            health_guide_content::health_guide_content_router(),
        )
        .nest(
            "/service_map_content",
            service_map_content::service_map_content_router(),
        )
        .nest("/policy_type", policy_type::policy_type_router())
        .nest("/policy_file", policy_file::policy_file_router());

    let app = Router::new()
        .nest("/api", api_router)
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();

    axum::serve(listener, app).await?;

    Ok(())
}
