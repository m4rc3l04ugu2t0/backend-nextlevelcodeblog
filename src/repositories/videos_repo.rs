use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    models::query::{CreateCategory, ResponseVideo, Video},
    Result,
};

use super::PostgresRepo;

#[async_trait]
pub trait VideosRepository: Send + Sync {
    async fn videos(&self) -> Result<Vec<Video>>;
    async fn create_video(
        &self,
        id: Uuid,
        title: &str,
        youtube_id: &str,
        duration: &str,
        views: Option<i32>,
    ) -> Result<()>;
    async fn update_video(
        &self,
        video_id: Uuid,
        title: Option<&str>,
        youtube_id: Option<&str>,
        duration: Option<&str>,
        views: Option<i32>,
    ) -> Result<()>;
    async fn add_category_to_video(&self, video_id: Uuid, category_id: Uuid) -> Result<()>;
    async fn delete_video(&self, video_id: Uuid) -> Result<()>;
    async fn remove_category_from_video(&self, video_id: Uuid, category_id: Uuid) -> Result<()>;
    async fn create_category(&self, category_id: Uuid, category: &str) -> Result<CreateCategory>;
    async fn delete_category(&self, category_id: Uuid) -> Result<()>;
    async fn get_video_by_youtube_id(&self, youtube_id: &str) -> Result<ResponseVideo>;
}

#[async_trait]
impl VideosRepository for PostgresRepo {
    async fn videos(&self) -> Result<Vec<Video>> {
        #[derive(sqlx::FromRow)]
        struct TempVideo {
            id: Uuid,
            title: String,
            youtube_id: String,
            duration: i32,
            views: i32,
            categories: Vec<String>,
        }

        let temp_videos = sqlx::query_as::<_, TempVideo>(
            r#"
            SELECT
                v.id,
                v.title,
                v.youtube_id,
                v.duration,
                v.views,
                COALESCE(array_agg(c.name) FILTER (WHERE c.name IS NOT NULL), '{}') as categories
            FROM videos v
            LEFT JOIN video_categories vc ON v.id = vc.video_id
            LEFT JOIN categories c ON vc.category_id = c.id
            GROUP BY v.id
            ORDER BY v.title ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let videos = temp_videos
            .into_iter()
            .map(|temp_video| Video {
                id: temp_video.id,
                title: temp_video.title,
                youtube_id: temp_video.youtube_id,
                duration: temp_video.duration.to_string(),
                views: temp_video.views,
                categories: temp_video.categories,
            })
            .collect();

        Ok(videos)
    }

    async fn create_video(
        &self,
        id: Uuid,
        title: &str,
        youtube_id: &str,
        duration: &str,
        views: Option<i32>,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO videos (id, title, youtube_id, duration, views)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(id)
        .bind(title)
        .bind(youtube_id)
        .bind(duration)
        .bind(views)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update_video(
        &self,
        video_id: Uuid,
        title: Option<&str>,
        youtube_id: Option<&str>,
        duration: Option<&str>,
        views: Option<i32>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE videos
            SET
                title = COALESCE($1, title),
                youtube_id = COALESCE($2, youtube_id),
                duration = COALESCE($3, duration),
                views = COALESCE($4, views)
            WHERE id = $5
            "#,
        )
        .bind(title)
        .bind(youtube_id)
        .bind(duration)
        .bind(views)
        .bind(video_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
    async fn add_category_to_video(&self, video_id: Uuid, category_id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO video_categories (video_id, category_id)
            VALUES ($1, $2);
            "#,
        )
        .bind(video_id)
        .bind(category_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn create_category(&self, category_id: Uuid, category: &str) -> Result<CreateCategory> {
        let category = sqlx::query_as::<_, CreateCategory>(
            r#"
            INSERT INTO categories (id, name)
            VALUES ($1, $2)
            RETURNING id, name;
            "#,
        )
        .bind(category_id)
        .bind(category)
        .fetch_one(&self.pool)
        .await?;

        Ok(category)
    }

    async fn delete_category(&self, category_id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM categories
            WHERE id = $1;
            "#,
        )
        .bind(category_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_video_by_youtube_id(&self, youtube_id: &str) -> Result<ResponseVideo> {
        let video = sqlx::query_as::<_, ResponseVideo>(
            r#"
            SELECT title, duration, views
            FROM videos
            WHERE youtube_id = $1
            "#,
        )
        .bind(youtube_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(video)
    }

    async fn delete_video(&self, video_id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM videos
            WHERE id = $1;
            "#,
        )
        .bind(video_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn remove_category_from_video(&self, video_id: Uuid, category_id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM video_categories
            WHERE video_id = $1 AND category_id = $2;
            "#,
        )
        .bind(video_id)
        .bind(category_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
