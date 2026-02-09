mod telegram;
mod web;

#[tokio::main]
async fn main() {
    web::routes::run_axum_server().await;
}
