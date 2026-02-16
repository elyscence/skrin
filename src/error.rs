use axum::{Json, extract::multipart::MultipartError, http::StatusCode, response::IntoResponse};
use serde_json::json;

// TODO: протестировать thiserror вместо этого используя встроенный from

pub enum AppError {
    MultipartError(MultipartError),
    NoFileProvided,
    InvalidInput,
    AlreadyExists,
    NoMimeType,
    IoError(std::io::Error),
    DatabaseError(sqlx::Error),
}

pub enum AuthError {
    NoTokenProvided,
    InvalidToken,
    DatabaseError(sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::NoFileProvided => (StatusCode::BAD_REQUEST, "No file provided"),
            AppError::InvalidInput => (StatusCode::BAD_REQUEST, "Invalid input"),
            AppError::AlreadyExists => (StatusCode::CONFLICT, "File already exists"),
            AppError::NoMimeType => (StatusCode::BAD_REQUEST, "MIME Type couldn't be determined"),
            AppError::IoError(err) => {
                tracing::error!("IO Error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Server error")
            }
            AppError::MultipartError(err) => {
                tracing::error!("Multipart Error: {}", err);
                (StatusCode::BAD_REQUEST, "Bad input")
            }
            AppError::DatabaseError(err) => {
                tracing::error!("Database Error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Server error")
            }
        };

        (
            status,
            Json(json!({
                "error": message,
                "success": false
            })),
        )
            .into_response()
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AuthError::NoTokenProvided => (StatusCode::UNAUTHORIZED, "No token provided"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
            AuthError::DatabaseError(err) => {
                tracing::error!("Database error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Server error")
            }
        };

        (
            status,
            Json(json!({
                "error": message,
                "success": false
            })),
        )
            .into_response()
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IoError(err)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::DatabaseError(err)
    }
}

impl From<MultipartError> for AppError {
    fn from(err: MultipartError) -> Self {
        AppError::MultipartError(err)
    }
}

impl From<sqlx::Error> for AuthError {
    fn from(err: sqlx::Error) -> Self {
        AuthError::DatabaseError(err)
    }
}
