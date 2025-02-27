use async_trait::async_trait;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::{
        news_post::{
            CommentWithAuthor, NewsPost, PostComment, PostCommentWithAuthor,
            PostCommentWithComments,
        },
        query::{CreateCategory, CreateVideo, ReturnVideo, Video, VideoDto},
    },
    Result,
};

use super::PostgresRepo;

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
    async fn get_comments_for_post(&self, post_id: &str) -> Result<Vec<PostCommentWithAuthor>>;
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
    async fn get_video_by_youtube_id(&self, youtube_id: &str) -> Result<ReturnVideo>;
}

#[async_trait]
impl NewsPostsRepository for PostgresRepo {
    async fn get_news_posts(&self) -> Result<Vec<NewsPost>> {
        let posts = sqlx::query_as!(
            NewsPost,
            r#"
            SELECT id, author_id, author_name, url, description, created_at FROM news_posts
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(posts)
    }

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

    async fn create_news_post(
        &self,
        url: &str,
        description: &str,
        author_id: &str,
        author_name: &str,
    ) -> Result<NewsPost> {
        let id = Uuid::now_v7();
        let author_id = Uuid::parse_str(&author_id).unwrap();

        let post = sqlx::query_as!(
            NewsPost,
            r#"
            INSERT INTO news_posts (id, url, description, author_id, author_name, created_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            RETURNING id, url, description, author_id, author_name, created_at
            "#,
            id,
            url,
            description,
            author_id,
            author_name
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
            RETURNING id, url, description, author_id, author_name, created_at
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
    async fn get_comments_for_post(&self, post_id: &str) -> Result<Vec<PostCommentWithAuthor>> {
        let post_id = Uuid::parse_str(post_id).unwrap();

        let comments = sqlx::query_as!(
            PostCommentWithAuthor,
            r#"
            SELECT pc.id, pc.content, pc.author_id, pc.created_at, u.name as author_name
            FROM post_comments pc
            JOIN users u ON u.id = pc.author_id
            WHERE pc.news_post_id = $1
            "#,
            post_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(comments)
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

        let comment = sqlx::query_as!(
            PostComment,
            r#"
            INSERT INTO post_comments (id, news_post_id, content, author_id, author_name)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, content, author_id, author_name, created_at
            "#,
            id,
            post_id,
            content,
            author_id,
            author_name
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(comment)
    }

    async fn update_comment(&self, comment_id: &str, content: Option<&str>) -> Result<PostComment> {
        let comment_id = Uuid::parse_str(comment_id).unwrap();

        let comment = sqlx::query_as!(
            PostComment,
            r#"
            UPDATE post_comments
            SET content = COALESCE($2, content)
            WHERE id = $1
            RETURNING id, content, author_id, author_name, created_at
            "#,
            comment_id,
            content
        )
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

    async fn get_posts_with_comments(&self, post_id: &str) -> Result<PostCommentWithComments> {
        let post_id = Uuid::parse_str(post_id).unwrap();

        let post_row = sqlx::query!(
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
                '[]'
            ) AS comments
        FROM post_data pd
        LEFT JOIN post_comments pc ON pd.id = pc.news_post_id
        LEFT JOIN users u2 ON pc.author_id = u2.id
        GROUP BY pd.id, pd.url, pd.description, pd.author_id, pd.author_name, pd.created_at
        "#,
            post_id
        )
        .fetch_one(&self.pool)
        .await?;

        let comments: Vec<CommentWithAuthor> = serde_json::from_value(
            post_row
                .comments
                .unwrap_or(serde_json::Value::Array(vec![])),
        )
        .unwrap();

        // Monta o objeto final
        Ok(PostCommentWithComments {
            id: post_row.id,
            url: post_row.url,
            description: post_row.description,
            author_id: post_row.author_id,
            author_name: post_row.author_name,
            created_at: post_row.created_at,
            comments,
        })
    }

    async fn get_all_posts_with_comments(&self) -> Result<Vec<PostCommentWithComments>> {
        let post_rows = sqlx::query!(
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
                    'authorId', pc.author_id,
                    'authorName', u2.name,
                    'createdAt', pc.created_at
                    )
                ) FILTER (WHERE pc.id IS NOT NULL),
                '[]'
            ) AS comments
        FROM post_data pd
        LEFT JOIN post_comments pc ON pd.id = pc.news_post_id
        LEFT JOIN users u2 ON pc.author_id = u2.id
        GROUP BY pd.id, pd.url, pd.description, pd.author_id, pd.author_name, pd.created_at
        "#
        )
        .fetch_all(&self.pool)
        .await?;

        let posts_with_comments: Vec<PostCommentWithComments> = post_rows
            .into_iter()
            .map(|post_row| {
                let comments: Vec<CommentWithAuthor> = serde_json::from_value(
                    post_row
                        .comments
                        .unwrap_or(serde_json::Value::Array(vec![])),
                )
                .unwrap();

                PostCommentWithComments {
                    id: post_row.id,
                    url: post_row.url,
                    description: post_row.description,
                    author_id: post_row.author_id,
                    author_name: post_row.author_name,
                    created_at: post_row.created_at,
                    comments,
                }
            })
            .collect();

        Ok(posts_with_comments)
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

    async fn get_video_by_youtube_id(&self, youtube_id: &str) -> Result<ReturnVideo> {
        let video = sqlx::query_as!(
            ReturnVideo,
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
