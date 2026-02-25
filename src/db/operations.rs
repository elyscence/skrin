use crate::models::{image::Image, stats::StatsResponse};
use sqlx::sqlite::SqlitePool;

pub async fn save_image(pool: &SqlitePool, image: &Image) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO images (id, filename, mime_type, size, uploaded_by, uploaded_at, views)
        VALUES (?, ?, ?, ?, ?, ?, ?)",
        image.id,
        image.filename,
        image.mime_type,
        image.size,
        image.uploaded_by,
        image.uploaded_at,
        image.views
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_user_images(pool: &SqlitePool, user_id: &str) -> Result<Vec<Image>, sqlx::Error> {
    sqlx::query_as!(
        Image,
        r#"SELECT
            id as "id!",
            filename as "filename!",
            mime_type as "mime_type!",
            size as "size!: i64",
            uploaded_by as "uploaded_by!",
            uploaded_at as "uploaded_at!",
            views as "views!: i64",
            deleted_at
        FROM images WHERE uploaded_by = ? AND deleted_at IS NULL ORDER BY uploaded_at DESC"#,
        user_id
    )
    .fetch_all(pool)
    .await
}

pub async fn is_token_valid(pool: &SqlitePool, token: &str) -> Result<Option<String>, sqlx::Error> {
    let user_id = sqlx::query_scalar!("SELECT user_id FROM tokens WHERE token = ?", token)
        .fetch_optional(pool)
        .await?;

    Ok(user_id)
}

pub async fn delete_image(
    pool: &SqlitePool,
    image_id: &str,
    user_id: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!("UPDATE images SET deleted_at = datetime('now') WHERE id = ? AND uploaded_by = ? AND deleted_at IS NULL", image_id, user_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn get_stats(pool: &SqlitePool) -> Result<StatsResponse, sqlx::Error> {
    sqlx::query_as!(
        StatsResponse,
        "SELECT
                COUNT(*) as total_images,
                COALESCE(SUM(size), 0) as total_size,
                COALESCE(SUM(views), 0) as total_views
            FROM images
            WHERE deleted_at IS NULL"
    )
    .fetch_one(pool)
    .await
}

pub async fn increment_views(pool: &SqlitePool, img_name: &str) -> Result<bool, sqlx::Error> {
    let image_name = img_name
        .rsplit_once(".")
        .map(|(name, _)| name)
        .unwrap_or(img_name);

    let result = sqlx::query!(
        "UPDATE images SET views = views + 1 WHERE filename = ?",
        image_name
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}
