use axum::{
    Router,
    extract::Multipart,
    response::Html,
    routing::{get, post},
};

//const DEFAULT_PATH = ""

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health", get(health))
        .route("/upload_form", get(show_form))
        .route("/upload", post(upload));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn upload(mut multipart: Multipart) {
    while let Some(field) = multipart.next_field().await.expect("Something went wrong") {
        let name = field.name().unwrap().to_string();
        let file_name = field
            .file_name()
            .expect("Cannot find file name")
            .to_string();
        let data = field.bytes();

        println!("Length of `{}`, name: {}", name, file_name);

        let save_file = std::fs::write(
            format!("./upload/{}", file_name),
            data.await.expect("Cannot upload file"),
        );
        match save_file {
            Ok(_) => println!("Saved {} succesfully!", file_name),
            Err(error) => println!("Something went wrong with {}: {}", file_name, error),
        }
    }
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
