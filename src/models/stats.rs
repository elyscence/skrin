use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StatsResponse {
    pub total_images: i64,
    pub total_size: i64,
    pub total_views: i64,
}
