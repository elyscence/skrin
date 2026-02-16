use crate::{
    db::operations::save_image,
    error::AppError,
    models::{image::Image, response::UploadResponse},
    state::AppState,
    utils::gen_id::generate_id,
};

use axum::{
    Json,
    body::Body,
    extract::{Multipart, Path, State},
    http::header,
    response::{Html, IntoResponse},
};

use tokio_util::io::ReaderStream;
use tracing::debug;
use uuid::Uuid;

// TODO: сделать impl IntoResponse for AppError, просто better handling ошибок

pub async fn upload(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, AppError> {
    let field = multipart
        .next_field()
        .await?
        .ok_or(AppError::NoFileProvided)?;

    let raw_name = field.file_name().ok_or(AppError::InvalidInput)?.to_string();

    let file_extension = raw_name.split(".").last().unwrap_or("png");
    let data = field.bytes().await?;

    let file_name = generate_id(5);

    let mime_guess = mime_guess::from_path(&raw_name)
        .first_raw()
        .ok_or(AppError::NoMimeType)?;

    debug!(
        "Uploading file: {}.{}, mime type: {}",
        file_name, file_extension, mime_guess
    );

    let formatted_path = format!(
        "{}/{}.{}",
        state.config.upload_path, file_name, file_extension
    );
    let url_path = format!(
        "{}/file/{}.{}",
        state.config.base_url, file_name, file_extension
    );

    if tokio::fs::try_exists(&formatted_path).await? {
        return Err(AppError::AlreadyExists);
    }

    let image_data = Image {
        id: Uuid::new_v4().to_string(),
        filename: file_name,
        mime_type: mime_guess.to_owned(),
        size: data.len() as i64,
        uploaded_by: String::from("anonymous"),
        uploaded_at: chrono::Utc::now().to_rfc3339(),
        views: 0,
        deleted_at: None,
    };

    save_image(&state.pool, &image_data).await?;

    tokio::fs::write(&formatted_path, data).await?;

    Ok(Json(UploadResponse {
        url: url_path,
        success: true,
    }))
}

pub async fn get_file(
    State(state): State<AppState>,
    Path(file_name): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let upload_dir = std::path::PathBuf::from(state.config.upload_path).canonicalize()?;

    let formatted_path = upload_dir.join(&file_name);

    if !formatted_path.starts_with(&upload_dir) {
        return Err(AppError::InvalidInput);
    }

    let file = tokio::fs::File::open(&formatted_path).await?;

    let content_type = mime_guess::from_path(&formatted_path)
        .first_raw()
        .ok_or(AppError::NoMimeType)?;

    debug!("Content type: {}", content_type);

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
