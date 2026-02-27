mod config;
mod db;
mod error;
mod middlewares;
mod models;
mod state;
mod utils;
mod web;

use std::time::Duration;

use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{delete, get, post},
};
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePool};

use state::AppState;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::services::ServeDir;
use tracing::info;

use crate::{
    config::Config, middlewares::auth::auth, utils::key_extractor::SmartIpExtractor, web::handlers,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let config = Config::from_env().unwrap_or_else(|e| {
        e.log();
        std::process::exit(1);
    });

    if !sqlx::Sqlite::database_exists(&config.database_url)
        .await
        .unwrap()
    {
        sqlx::Sqlite::create_database(&config.database_url)
            .await
            .unwrap();
    }

    let pool = SqlitePool::connect(&config.database_url)
        .await
        .unwrap_or_else(|e| {
            tracing::error!("Database connection failed: {}", e);
            std::process::exit(1);
        });

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to connect to database");

    let governor_conf = GovernorConfigBuilder::default()
        .per_second(2)
        .burst_size(5)
        .key_extractor(SmartIpExtractor)
        .finish()
        .unwrap();

    let governor_limiter = governor_conf.limiter().clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            governor_limiter.retain_recent();
        }
    });

    let state = AppState::new(pool, config.clone());

    let upload_router = Router::new()
        .route("/api/upload", post(handlers::upload))
        .layer(GovernorLayer::new(governor_conf))
        .route_layer(from_fn_with_state(state.clone(), auth));

    let protected_router = Router::new()
        .route("/api/my", get(handlers::my_images))
        .route("/api/{image_id}", delete(handlers::delete_image_route))
        .route_layer(from_fn_with_state(state.clone(), auth));

    let app = Router::new()
        .merge(upload_router)
        .merge(protected_router)
        .route("/health", get(handlers::health))
        .route("/upload_form", get(handlers::show_form))
        .route("/file/{file_name}", get(handlers::get_file))
        .route("/api/stats", get(handlers::get_stats_route))
        .fallback_service(ServeDir::new("frontend"))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&config.bind_address)
        .await
        .unwrap();
    info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
