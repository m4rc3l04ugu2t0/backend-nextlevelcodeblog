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
        let videos = sqlx::query_as!(
            Video,
            r#"
            SELECT
                v.id,
                v.title,
                v.youtube_id,
                v.duration,
                v.views,
                COALESCE(array_agg(c.name) FILTER (WHERE c.name IS NOT NULL), '{}') as "categories: Vec<String>"
            FROM videos v
            LEFT JOIN video_categories vc ON v.id = vc.video_id
            LEFT JOIN categories c ON vc.category_id = c.id
            GROUP BY v.id
            ORDER BY v.title ASC;
            "#
        )
        .fetch_all(&self.pool)
        .await?;

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
        sqlx::query!(
            "INSERT INTO videos (id, title, youtube_id, duration, views)
             VALUES ($1, $2, $3, $4, $5)",
            id,
            title,
            youtube_id,
            duration,
            views
        )
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
        sqlx::query!(
            r#"
            UPDATE videos
            SET
                title = COALESCE($1, title),
                youtube_id = COALESCE($2, youtube_id),
                duration = COALESCE($3, duration),
                views = COALESCE($4, views)
            WHERE id = $5
            "#,
            title,
            youtube_id,
            duration,
            views,
            video_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
    async fn add_category_to_video(&self, video_id: Uuid, category_id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO video_categories (video_id, category_id)
            VALUES ($1, $2);
            "#,
            video_id,
            category_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn create_category(&self, category_id: Uuid, category: &str) -> Result<CreateCategory> {
        let category = sqlx::query_as!(
            CreateCategory,
            r#"
            INSERT INTO categories (id, name)
            VALUES ($1, $2)
            RETURNING id, name;
            "#,
            category_id,
            category
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(category)
    }

    async fn delete_category(&self, category_id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM categories
            WHERE id = $1;
            "#,
            category_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_video_by_youtube_id(&self, youtube_id: &str) -> Result<ResponseVideo> {
        let video = sqlx::query_as!(
            ResponseVideo,
            r#"
            SELECT title, duration, views
            FROM videos
            WHERE youtube_id = $1
            "#,
            youtube_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(video)
    }

    async fn delete_video(&self, video_id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM videos
            WHERE id = $1;
            "#,
            video_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn remove_category_from_video(&self, video_id: Uuid, category_id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM video_categories
            WHERE video_id = $1 AND category_id = $2;
            "#,
            video_id,
            category_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
