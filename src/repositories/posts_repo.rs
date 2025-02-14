use async_trait::async_trait;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{models::posts::Post, Result};

use super::PostgresRepo;

// Ajustando a trait para incluir `user_id`
#[async_trait]
pub trait PostsRepository: Sync + Send {
    async fn get_posts(&self) -> Result<Vec<Post>>;
    async fn create_post(
        &self,
        user_id: &str,
        name: &str,
        title: &str,
        description: &str,
        cover_image: &str,
    ) -> Result<Post>;
    async fn update_post(
        &self,
        post_id: &str,
        name: Option<&str>,
        title: Option<&str>,
        description: Option<&str>,
        cover_image: Option<&str>,
    ) -> Result<Post>;
    async fn delete_post(&self, post_id: &str) -> Result<()>;
}

#[async_trait]
impl PostsRepository for PostgresRepo {
    async fn get_posts(&self) -> Result<Vec<Post>> {
        let posts = sqlx::query_as!(
            Post,
            r#"
            SELECT id, user_id, name, title, description, cover_image, created_at FROM posts
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(posts)
    }

    async fn create_post(
        &self,
        user_id: &str,
        name: &str,
        title: &str,
        description: &str,
        cover_image: &str,
    ) -> Result<Post> {
        let user_id = Uuid::parse_str(&user_id).unwrap();
        let id = Uuid::now_v7();

        let post = sqlx::query_as!(
            Post,
            r#"
          INSERT INTO posts (id, user_id, name, title, description, cover_image)
          VALUES ($1, $2, $3, $4, $5, $6)
          RETURNING id, user_id, name, title, description, cover_image, created_at
          "#,
            id,
            user_id,
            name,
            title,
            description,
            cover_image,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(post)
    }

    async fn update_post(
        &self,
        post_id: &str,
        name: Option<&str>,
        title: Option<&str>,
        description: Option<&str>,
        cover_image: Option<&str>,
    ) -> Result<Post> {
        let post_id = Uuid::parse_str(post_id).unwrap();

        let post = sqlx::query_as!(
            Post,
            r#"
            UPDATE posts
            SET name = COALESCE($2, name),
                title = COALESCE($3, title),
                description = COALESCE($4, description),
                cover_image = COALESCE($5, cover_image)
            WHERE id = $1
            RETURNING id, user_id, name, title, description, cover_image, created_at
            "#,
            post_id,
            name,
            title,
            description,
            cover_image
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(post)
    }

    async fn delete_post(&self, post_id: &str) -> Result<()> {
        let post_id = Uuid::parse_str(post_id).unwrap();

        sqlx::query!(
            r#"
            DELETE FROM posts WHERE id = $1
            "#,
            post_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
