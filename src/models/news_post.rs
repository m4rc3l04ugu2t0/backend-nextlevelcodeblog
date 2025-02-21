use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow, Clone)]
pub struct NewsPost {
    pub id: Uuid,
    pub url: String,
    #[serde(rename = "authorId")]
    pub author_id: Uuid,
    #[serde(rename = "authorName")]
    pub author_name: String,
    pub description: String,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct PostCommentWithComments {
    pub id: Uuid,
    pub url: String,
    pub description: String,
    #[serde(rename = "authorId")]
    pub author_id: Uuid,
    #[serde(rename = "authorName")]
    pub author_name: String,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    pub comments: Vec<CommentWithAuthor>,
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct CommentWithAuthor {
    pub id: Uuid,
    pub content: String,
    #[serde(rename = "authorId")]
    pub author_id: Uuid,
    #[serde(rename = "authorName")]
    pub author_name: String,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow, Clone)]
pub struct PostComment {
    pub id: String,
    pub content: String,
    #[serde(rename = "authorId")]
    pub author_id: Uuid,
    #[serde(rename = "authorName")]
    pub author_name: String,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow, Clone)]
pub struct PostCommentDto {
    pub id: String,
    pub content: String,
    #[serde(rename = "authorName")]
    pub author_name: String,
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow, Clone)]
pub struct UpdatePostCommentDto {
    pub content: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow, Clone)]
pub struct CreateNewsPostDto {
    pub url: String,
    pub description: String,
    #[serde(rename = "authorName")]
    pub author_name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNewsPost {
    pub url: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow, Clone)]
pub struct PostCommentWithAuthor {
    pub id: Uuid,
    pub content: String,
    #[serde(rename = "authorId")]
    pub author_id: Uuid,
    #[serde(rename = "authorName")]
    pub author_name: String, // Apenas nessa struct
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
}
