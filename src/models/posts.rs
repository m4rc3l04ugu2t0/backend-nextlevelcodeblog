use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow, Clone)]
pub struct Post {
    pub id: Uuid,
    pub user_id: String,
    pub name: String,
    pub title: String,
    pub description: String,
    pub cover_image: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePostDto {
    pub name: String,
    pub title: String,
    pub user_id: String,
    pub description: String,
    pub cover_image: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePost {
    pub name: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub cover_image: Option<String>,
}
