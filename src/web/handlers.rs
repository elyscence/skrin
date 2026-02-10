use crate::utils::gen_id::generate_id;
use axum::{
    Json,
    body::Body,
    extract::{Multipart, Path},
    http::header,
    response::{Html, IntoResponse},
};

use tokio_util::io::ReaderStream;
use tracing::debug;

use crate::error::AppError;
use crate::models::response::UploadResponse;

const UPLOAD_PATH: &str = "./upload";

// TODO: сделать impl IntoResponse for AppError, просто better handling ошибок

pub async fn upload(mut multipart: Multipart) -> Result<Json<UploadResponse>, AppError> {
    let field = multipart
        .next_field()
        .await?
        .ok_or(AppError::NoFileProvided)?;

    let raw_name = field.file_name().ok_or(AppError::InvalidInput)?.to_string();

    let file_extension = raw_name.split(".").last().unwrap_or("png");

    let data = field.bytes().await?;

    let file_name = generate_id(5);

    debug!("Uploading file: {}.{}", file_name, file_extension);

    let formatted_path = format!("{}/{}.{}", UPLOAD_PATH, file_name, file_extension);
    let url_path = format!(
        "http://localhost:3000/file/{}.{}",
        file_name, file_extension
    );

    if tokio::fs::try_exists(&formatted_path).await? {
        return Err(AppError::AlreadyExists);
    }

    tokio::fs::write(&formatted_path, data).await?;

    Ok(Json(UploadResponse {
        url: url_path,
        success: true,
    }))
}

pub async fn get_file(Path(file_name): Path<String>) -> Result<impl IntoResponse, AppError> {
    let upload_dir = std::path::PathBuf::from(UPLOAD_PATH).canonicalize()?;

    let formatted_path = upload_dir.join(&file_name);

    if !formatted_path.starts_with(&upload_dir) {
        return Err(AppError::InvalidInput);
    }

    let file = tokio::fs::File::open(&formatted_path).await?;

    let content_type = mime_guess::from_path(&formatted_path)
        .first_raw()
        .ok_or(AppError::NoMimeType)?;

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
