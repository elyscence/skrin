mod config;
mod db;
mod error;
mod middlewares;
mod models;
mod state;
mod utils;
mod web;

use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{get, post},
};
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePool};

use state::AppState;
use tracing::info;

use crate::{config::Config, middlewares::auth::auth, web::handlers};

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

    let state = AppState::new(pool, config.clone());

    let protected_router = Router::new()
        .route("/upload", post(handlers::upload))
        .route_layer(from_fn_with_state(state.clone(), auth));

    let app = Router::new()
        .merge(protected_router)
        .route("/health", get(handlers::health))
        .route("/upload_form", get(handlers::show_form))
        .route("/file/{file_name}", get(handlers::get_file))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&config.bind_address)
        .await
        .unwrap();
    info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
