use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow, Clone)]
pub struct Post {
    pub id: Uuid,
    pub user_id: String,
    pub title: String,
    pub description: String,
    pub cover_image: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePostDto {
    pub title: String,
    pub user_id: String,
    pub description: String,
    pub cover_image: String,
}
