mod web;

use axum::{
    Router,
    routing::{get, post},
};

use tracing::debug;

use crate::web::handlers;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let app = Router::new()
        .route("/health", get(handlers::health))
        .route("/upload_form", get(handlers::show_form))
        .route("/file/{file_name}", get(handlers::get_file))
        .route("/upload", post(handlers::upload));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    debug!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
