use super::PostgresRepo;
use crate::{
    models::news_post::{CommentWithAuthor, NewsPost, PostComment, PostCommentWithComments},
    Error, Result,
};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait NewsPostsRepository: Sync + Send {
    async fn get_news_posts(&self) -> Result<Vec<NewsPost>>;
    async fn create_news_post(
        &self,
        url: &str,
        description: &str,
        author_id: &str,
        author_name: &str,
    ) -> Result<NewsPost>;
    async fn update_news_post(
        &self,
        post_id: &str,
        url: Option<&str>,
        description: Option<&str>,
    ) -> Result<NewsPost>;
    async fn delete_news_post(&self, post_id: &str) -> Result<()>;
    async fn create_comment(
        &self,
        post_id: &str,
        content: &str,
        author_id: &str,
        author_name: &str,
    ) -> Result<PostComment>;
    async fn update_comment(&self, comment_id: &str, content: Option<&str>) -> Result<PostComment>;
    async fn delete_comment(&self, comment_id: &str) -> Result<()>;
    async fn get_posts_with_comments(&self, post_id: &str) -> Result<PostCommentWithComments>;
    async fn get_all_posts_with_comments(&self) -> Result<Vec<PostCommentWithComments>>;
}

#[async_trait]
impl NewsPostsRepository for PostgresRepo {
    async fn get_news_posts(&self) -> Result<Vec<NewsPost>> {
        let posts = sqlx::query_as::<_, NewsPost>(
            r#"
            SELECT id, author_id, author_name, url, description, created_at FROM news_posts
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(posts)
    }

    async fn create_news_post(
        &self,
        url: &str,
        description: &str,
        author_id: &str,
        author_name: &str,
    ) -> Result<NewsPost> {
        let id = Uuid::now_v7();
        let author_id = Uuid::parse_str(author_id).unwrap();

        let post = sqlx::query_as::<_, NewsPost>(
            r#"
            INSERT INTO news_posts (id, url, description, author_id, author_name, created_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            RETURNING id, url, description, author_id, author_name, created_at
            "#,
        )
        .bind(id)
        .bind(url)
        .bind(description)
        .bind(author_id)
        .bind(author_name)
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

        let post = sqlx::query_as::<_, NewsPost>(
            r#"
            UPDATE news_posts
            SET url = COALESCE($2, url),
                description = COALESCE($3, description)
            WHERE id = $1
            RETURNING id, url, description, author_id, author_name, created_at
            "#,
        )
        .bind(post_id)
        .bind(url)
        .bind(description)
        .fetch_one(&self.pool)
        .await?;

        Ok(post)
    }

    async fn create_comment(
        &self,
        post_id: &str,
        content: &str,
        author_id: &str,
        author_name: &str,
    ) -> Result<PostComment> {
        let post_id = Uuid::parse_str(post_id).unwrap();
        let author_id = Uuid::parse_str(author_id).unwrap();

        let id = Uuid::now_v7();

        let comment = sqlx::query_as::<_, PostComment>(
            r#"
            INSERT INTO post_comments (id, news_post_id, content, author_id, author_name)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, content, author_id, author_name, created_at
            "#,
        )
        .bind(id)
        .bind(post_id)
        .bind(content)
        .bind(author_id)
        .bind(author_name)
        .fetch_one(&self.pool)
        .await?;

        Ok(comment)
    }

    async fn update_comment(&self, comment_id: &str, content: Option<&str>) -> Result<PostComment> {
        let comment_id = Uuid::parse_str(comment_id).unwrap();

        let comment = sqlx::query_as::<_, PostComment>(
            r#"
            UPDATE post_comments
            SET content = COALESCE($2, content)
            WHERE id = $1
            RETURNING id, content, author_id, author_name, created_at
            "#,
        )
        .bind(comment_id)
        .bind(content)
        .fetch_one(&self.pool)
        .await?;

        Ok(comment)
    }

    async fn delete_comment(&self, comment_id: &str) -> Result<()> {
        let comment_id = Uuid::parse_str(comment_id).unwrap();

        sqlx::query!(
            r#"
            DELETE FROM post_comments WHERE id = $1
            "#,
            comment_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_news_post(&self, post_id: &str) -> Result<()> {
        let post_id = Uuid::parse_str(post_id).unwrap();

        sqlx::query(
            r#"
            DELETE FROM news_posts WHERE id = $1
            "#,
        )
        .bind(post_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_posts_with_comments(&self, post_id: &str) -> Result<PostCommentWithComments> {
        let post_id = Uuid::parse_str(post_id).unwrap();

        #[derive(sqlx::FromRow)]
        struct TempPost {
            id: Uuid,
            url: String,
            description: String,
            author_id: Uuid,
            author_name: String,
            created_at: chrono::DateTime<chrono::Utc>,
            comments: serde_json::Value,
        }

        let temp_post = sqlx::query_as::<_, TempPost>(
            r#"
            WITH post_data AS (
                SELECT
                    np.id,
                    np.url,
                    np.description,
                    np.author_id,
                    u.name as author_name,
                    np.created_at
                FROM news_posts np
                JOIN users u ON np.author_id = u.id
                WHERE np.id = $1
            )
            SELECT
                pd.id,
                pd.url,
                pd.description,
                pd.author_id,
                pd.author_name,
                pd.created_at,
                COALESCE(
                    json_agg(
                        json_build_object(
                            'id', pc.id,
                            'content', pc.content,
                            'author_id', pc.author_id,
                            'author_name', u2.name,
                            'created_at', pc.created_at
                        )
                    ) FILTER (WHERE pc.id IS NOT NULL),
                    '[]'::json
                ) AS comments
            FROM post_data pd
            LEFT JOIN post_comments pc ON pd.id = pc.news_post_id
            LEFT JOIN users u2 ON pc.author_id = u2.id
            GROUP BY pd.id, pd.url, pd.description, pd.author_id, pd.author_name, pd.created_at
            "#,
        )
        .bind(post_id)
        .fetch_one(&self.pool)
        .await?;

        let comments: Vec<CommentWithAuthor> =
            serde_json::from_value(temp_post.comments).map_err(|_| Error::NotFound)?;

        Ok(PostCommentWithComments {
            id: temp_post.id,
            url: temp_post.url,
            description: temp_post.description,
            author_id: temp_post.author_id,
            author_name: temp_post.author_name,
            created_at: temp_post.created_at,
            comments,
        })
    }

    async fn get_all_posts_with_comments(&self) -> Result<Vec<PostCommentWithComments>> {
        #[derive(sqlx::FromRow)]
        struct TempPost {
            id: Uuid,
            url: String,
            description: String,
            author_id: Uuid,
            author_name: String,
            created_at: chrono::DateTime<chrono::Utc>,
            comments: serde_json::Value,
        }

        let temp_posts = sqlx::query_as::<_, TempPost>(
            r#"
            WITH post_data AS (
                SELECT
                    np.id,
                    np.url,
                    np.description,
                    np.author_id,
                    u.name as author_name,
                    np.created_at
                FROM news_posts np
                JOIN users u ON np.author_id = u.id
            )
            SELECT
                pd.id,
                pd.url,
                pd.description,
                pd.author_id,
                pd.author_name,
                pd.created_at,
                COALESCE(
                    json_agg(
                        json_build_object(
                            'id', pc.id,
                            'content', pc.content,
                            'author_id', pc.author_id,
                            'author_name', u2.name,
                            'created_at', pc.created_at
                        )
                    ) FILTER (WHERE pc.id IS NOT NULL),
                    '[]'::json
                ) AS comments
            FROM post_data pd
            LEFT JOIN post_comments pc ON pd.id = pc.news_post_id
            LEFT JOIN users u2 ON pc.author_id = u2.id
            GROUP BY pd.id, pd.url, pd.description, pd.author_id, pd.author_name, pd.created_at
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let posts_with_comments = temp_posts
            .into_iter()
            .map(|temp_post| {
                let comments: Vec<CommentWithAuthor> =
                    serde_json::from_value(temp_post.comments).unwrap();

                PostCommentWithComments {
                    id: temp_post.id,
                    url: temp_post.url,
                    description: temp_post.description,
                    author_id: temp_post.author_id,
                    author_name: temp_post.author_name,
                    created_at: temp_post.created_at,
                    comments,
                }
            })
            .collect();

        Ok(posts_with_comments)
    }
}
