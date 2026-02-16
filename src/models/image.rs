use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Image {
    pub id: String,
    pub filename: String,
    pub mime_type: String,
    pub size: i64,
    pub uploaded_by: String,
    pub uploaded_at: String,
    pub views: i64,
    pub deleted_at: Option<String>,
}
