use async_trait::async_trait;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{models::news_post::NewsPost, Result};

use super::PostgresRepo;

#[async_trait]
pub trait NewsPostsRepository: Sync + Send {
    async fn get_news_posts(&self) -> Result<Vec<NewsPost>>;
    async fn create_news_post(&self, url: &str, description: &str) -> Result<NewsPost>;
    async fn update_news_post(
        &self,
        post_id: &str,
        url: Option<&str>,
        description: Option<&str>,
    ) -> Result<NewsPost>;
    async fn delete_news_post(&self, post_id: &str) -> Result<()>;
}

#[async_trait]
impl NewsPostsRepository for PostgresRepo {
    async fn get_news_posts(&self) -> Result<Vec<NewsPost>> {
        let posts = sqlx::query_as!(
            NewsPost,
            r#"
            SELECT id, url, description, created_at FROM news_posts
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(posts)
    }

    async fn create_news_post(&self, url: &str, description: &str) -> Result<NewsPost> {
        let id = Uuid::now_v7();

        let post = sqlx::query_as!(
            NewsPost,
            r#"
          INSERT INTO news_posts (id, url, description)
          VALUES ($1, $2, $3)
          RETURNING id, url, description, created_at
          "#,
            id,
            url,
            description
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(post)
    }

    async fn update_news_post(
        &self,
        post_id: &str,
        url: Option<&str>,
        description: Option<&str>,
    ) -> Result<NewsPost> {
        let post_id = Uuid::parse_str(post_id).unwrap();

        let post = sqlx::query_as!(
            NewsPost,
            r#"
            UPDATE news_posts
            SET url = COALESCE($2, url),
                description = COALESCE($3, description)
            WHERE id = $1
            RETURNING id, url, description, created_at
            "#,
            post_id,
            url,
            description
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(post)
    }

    async fn delete_news_post(&self, post_id: &str) -> Result<()> {
        let post_id = Uuid::parse_str(post_id).unwrap();

        sqlx::query!(
            r#"
            DELETE FROM news_posts WHERE id = $1
            "#,
            post_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
