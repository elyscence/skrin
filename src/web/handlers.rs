use axum::{
    body::Body,
    extract::{Multipart, Path},
    http::{StatusCode, header},
    response::{Html, IntoResponse},
};
use tokio_util::io::ReaderStream;
use tracing::debug;

const UPLOAD_PATH: &str = "./upload";

pub async fn upload(mut multipart: Multipart) -> impl IntoResponse {
    while let Some(field) = multipart.next_field().await.expect("Something went wrong") {
        let name = field.name().unwrap().to_string();
        let file_name = field
            .file_name()
            .expect("Cannot find file name")
            .to_string();
        let data = field.bytes();

        debug!("Length of `{}`, name: {}", name, file_name);

        let formatted_path = format!("{}/{}", UPLOAD_PATH, file_name);

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

pub async fn get_file(
    Path(file_name): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let upload_dir = std::path::PathBuf::from(UPLOAD_PATH)
        .canonicalize()
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Upload dir error".to_string(),
            )
        })?;

    let formatted_path = upload_dir.join(&file_name);

    if !formatted_path.starts_with(&upload_dir) {
        return Err((StatusCode::BAD_REQUEST, "Invalid file path".to_string()));
    }

    let file = tokio::fs::File::open(&formatted_path)
        .await
        .map_err(|err| (StatusCode::NOT_FOUND, format!("File not found: {}", err)))?;

    let content_type = match mime_guess::from_path(&formatted_path).first_raw() {
        Some(mime) => mime,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                "MIME Type couldn't be determined".to_string(),
            ));
        }
    };

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let headers = [
        (header::CONTENT_TYPE, content_type),
        (header::CONTENT_DISPOSITION, "inline"),
    ];

    Ok((headers, body))
}

pub async fn show_form() -> Html<&'static str> {
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

pub async fn health() -> &'static str {
    "alive"
}
