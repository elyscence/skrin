use axum::{
    Router,
    extract::Multipart,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
};
use tracing::debug;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new()
        .route("/health", get(health))
        .route("/upload_form", get(show_form))
        .route("/upload", post(upload));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    debug!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn upload(mut multipart: Multipart) -> impl IntoResponse {
    while let Some(field) = multipart.next_field().await.expect("Something went wrong") {
        let name = field.name().unwrap().to_string();
        let file_name = field
            .file_name()
            .expect("Cannot find file name")
            .to_string();
        let data = field.bytes();

        debug!("Length of `{}`, name: {}", name, file_name);

        let formatted_path = format!("./upload/{}", file_name);

        if let Ok(true) = tokio::fs::try_exists(&formatted_path).await {
            return (StatusCode::BAD_REQUEST, "Upload failed");
        }

        if let Err(_) =
            tokio::fs::write(&formatted_path, data.await.expect("Failed to upload file")).await
        {
            return (StatusCode::BAD_REQUEST, "Upload failed");
        }
    }

    (StatusCode::CREATED, "Image uploaded")
}

async fn show_form() -> Html<&'static str> {
    Html(
        r#"
        <!doctype html>
        <html>
            <head></head>
            <body>
                <form action="/upload" method="post" enctype="multipart/form-data">
                    <label>
                        Upload file:
                        <input type="file" name="file" multiple>
                    </label>

                    <input type="submit" value="Upload files">
                </form>
            </body>
        </html>
        "#,
    )
}

async fn health() -> &'static str {
    "alive"
}
