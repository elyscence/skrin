use crate::{
    db::operations::{
        delete_image, get_image_by_id, get_stats, get_user_images, increment_views, save_image,
    },
    error::AppError,
    middlewares::auth::AuthUser,
    models::{file_query::FileQuery, image::Image, response::UploadResponse, stats::StatsResponse},
    state::AppState,
    utils::{detect_format::detect_image_format, gen_id::generate_id},
};

use axum::{
    Extension, Json,
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::header,
    response::{Html, IntoResponse},
};

use serde_json::json;
use tokio_util::io::ReaderStream;
use tracing::debug;
use uuid::Uuid;

const MAX_SIZE: usize = 10 * 1024 * 1024;

// TODO: strip'ать EXIF при загрузке, выбрать kamadak-exif/rexiv2
pub async fn upload(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, AppError> {
    let field = multipart
        .next_field()
        .await?
        .ok_or(AppError::NoFileProvided)?;

    let raw_name = field.file_name().ok_or(AppError::InvalidInput)?.to_string();

    if raw_name.contains("/") || raw_name.contains("..") {
        return Err(AppError::InvalidInput);
    }

    let data = field.bytes().await?;

    let real_mime = detect_image_format(&data).ok_or(AppError::InvalidInput)?;

    let mime_type = real_mime.to_string();

    let file_extension = match real_mime {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/webp" => "webp",
        _ => return Err(AppError::InvalidInput),
    };

    if data.len() > MAX_SIZE {
        return Err(AppError::InvalidInput);
    }

    let file_name = generate_id(5);

    debug!(
        "Uploading file: {}.{}, mime type: {}",
        file_name, file_extension, mime_type
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
        filename: format!("{}.{}", file_name, file_extension),
        mime_type: mime_type.to_owned(),
        size: data.len() as i64,
        uploaded_by: auth_user.user_id,
        uploaded_at: chrono::Utc::now().to_rfc3339(),
        views: 0,
        deleted_at: None,
    };

    tokio::fs::write(&formatted_path, data).await?;

    save_image(&state.pool, &image_data).await?;

    Ok(Json(UploadResponse {
        url: url_path,
        success: true,
    }))
}

// TODO: Если файла нет в upload, то установить ему deleted_at now
pub async fn get_file(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<FileQuery>,
) -> Result<impl IntoResponse, AppError> {
    let file_name = get_image_by_id(&state.pool, &id)
        .await?
        .ok_or(AppError::NotFound)?;

    let upload_dir = std::path::PathBuf::from(&state.config.upload_path).canonicalize()?;

    let formatted_path = upload_dir.join(&file_name);

    tracing::debug!("Formatted path: {}", formatted_path.to_string_lossy());

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

    if query.thumb != Some(true) {
        if let Err(e) = increment_views(&state.pool, &id).await {
            tracing::warn!("Failed to increment views for {}: {}", id, e);
        }
    }

    Ok((headers, body))
}

pub async fn my_images(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<Vec<Image>>, AppError> {
    let user_images = get_user_images(&state.pool, &auth_user.user_id).await?;
    Ok(Json(user_images))
}

pub async fn delete_image_route(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(image_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    delete_image(&state.pool, &image_id, &auth_user.user_id)
        .await?
        .then_some(Json(json!({ "success": true })))
        .ok_or(AppError::NotFound)
}

pub async fn get_stats_route(
    State(state): State<AppState>,
) -> Result<Json<StatsResponse>, AppError> {
    Ok(Json(get_stats(&state.pool).await?))
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
