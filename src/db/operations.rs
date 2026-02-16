use crate::models::image::Image;
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

pub async fn is_token_valid(pool: &SqlitePool, token: &str) -> Result<bool, sqlx::Error> {
    let exists = sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM tokens WHERE token = ?)", token)
        .fetch_one(pool)
        .await?;

    Ok(exists == 1)
}
