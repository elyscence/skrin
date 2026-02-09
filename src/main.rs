mod utils;
mod web;

use axum::{
    Router,
    routing::{get, post},
};
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePool};

use tracing::info;

use crate::web::handlers;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://skrin.db".to_string());

    if !sqlx::Sqlite::database_exists(&db_url).await.unwrap() {
        sqlx::Sqlite::create_database(&db_url).await.unwrap();
    }

    let pool = SqlitePool::connect(&db_url)
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to connect to database");

    let app = Router::new()
        .route("/health", get(handlers::health))
        .route("/upload_form", get(handlers::show_form))
        .route("/file/{file_name}", get(handlers::get_file))
        .route("/upload", post(handlers::upload));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
