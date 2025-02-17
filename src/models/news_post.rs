use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow, Clone)]
pub struct NewsPost {
    pub id: Uuid,
    pub url: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateNewsPostDto {
    pub url: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNewsPost {
    pub url: Option<String>,
    pub description: Option<String>,
}
