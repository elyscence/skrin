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

pub async fn get_image_by_id(pool: &SqlitePool, id: &str) -> Result<Option<String>, sqlx::Error> {
    let image_id = sqlx::query_scalar!(
        "SELECT filename as \"filename!\" FROM images WHERE id = ? AND deleted_at IS NULL",
        id
    )
    .fetch_optional(pool)
    .await?;

    Ok(image_id)
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
    let result: sqlx::sqlite::SqliteQueryResult = sqlx::query!("UPDATE images SET deleted_at = datetime('now') WHERE id = ? AND uploaded_by = ? AND deleted_at IS NULL", image_id, user_id)
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

pub async fn increment_views(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let result: sqlx::sqlite::SqliteQueryResult =
        sqlx::query!("UPDATE images SET views = views + 1 WHERE id = ?", id)
            .execute(pool)
            .await?;

    Ok(result.rows_affected() > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_db() -> SqlitePool {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        pool
    }

    fn test_image(id: &str, user: &str) -> Image {
        Image {
            id: String::from(id),
            filename: format!("{}.png", id),
            mime_type: String::from("image/png"),
            size: 1111,
            uploaded_by: String::from(user),
            uploaded_at: String::from("2026-01-01"),
            views: 0,
            deleted_at: None,
        }
    }

    #[tokio::test]
    async fn upload_test() {
        let pool = setup_db().await;

        let test_image_data = test_image("test", "test_user");

        let result = save_image(&pool, &test_image_data).await;
        assert!(
            result.is_ok(),
            "save_image должен успешно сохранить картинку"
        );

        let user_images = get_user_images(&pool, "test_user").await.unwrap();
        assert_eq!(
            user_images.len(),
            1,
            "должна быть ровно одна картинка у test_user"
        );
        assert_eq!(
            user_images[0].filename, "test.png",
            "первая картинка должна быть с названием test.png"
        );
    }

    #[tokio::test]
    async fn increment_test() {
        let pool = setup_db().await;

        let test_image_data = test_image("views-test-id", "test_user");
        save_image(&pool, &test_image_data).await.unwrap();

        let result = increment_views(&pool, "views-test-id").await.unwrap();
        assert!(
            result,
            "increment_views должен вернуть true для существующей картинки"
        );

        let result = increment_views(&pool, "test").await.unwrap();
        assert!(
            !result,
            "просмотры не должны увеличиваться на несуществующую картинку"
        );
    }

    #[tokio::test]
    async fn delete_test() {
        let pool = setup_db().await;

        let test_image_data = test_image("test-id", "user_a");
        save_image(&pool, &test_image_data).await.unwrap();

        let result = delete_image(&pool, "test-id", "user_b").await.unwrap();

        assert!(!result, "user_b не должен мочь удалить картинку user_a");

        let result = delete_image(&pool, "test-id", "user_a").await.unwrap();

        assert!(result, "user_a должен мочь удалить свою картинку");
    }

    #[tokio::test]
    async fn soft_delete_test() {
        let pool = setup_db().await;
        save_image(&pool, &test_image("soft-delete-id", "user_a"))
            .await
            .unwrap();
        delete_image(&pool, "soft-delete-id", "user_a")
            .await
            .unwrap();

        let images = get_user_images(&pool, "user_a").await.unwrap();
        assert_eq!(images.len(), 0, "удалённая картинка не должна быть видна");
    }

    #[tokio::test]
    async fn get_filename_by_id_test() {
        let pool = setup_db().await;
        save_image(&pool, &test_image("filename-test-id", "user_a"))
            .await
            .unwrap();

        let filename = get_image_by_id(&pool, "filename-test-id").await.unwrap();
        assert_eq!(
            filename,
            Some(String::from("filename-test-id.png")),
            "должен вернуть filename по id"
        );

        let not_found = get_image_by_id(&pool, "nonexistent").await.unwrap();
        assert!(not_found.is_none(), "несуществующий id должен вернуть None");
    }
}
