use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Serialize, Deserialize, Validate)]
pub struct VerifyEmailQueryDto {
    #[validate(length(min = 1, message = "Token is required."))]
    pub token: String,
}

#[derive(Debug, sqlx::FromRow, Deserialize, Serialize)]
pub struct Video {
    pub id: Uuid,
    pub title: String,
    pub youtube_id: String,
    pub duration: String,
    pub views: i32,
    pub categories: Vec<String>, // Categorias como um vetor de strings
}

#[derive(Debug, sqlx::FromRow, Deserialize, Serialize)]
pub struct CreateVideo {
    pub id: uuid::Uuid, // Tipo correto para UUID
    pub title: String,
    pub youtube_id: String,
    pub duration: String,
    pub views: i32,
}

#[derive(Debug, sqlx::FromRow, Deserialize, Serialize)]
pub struct ResponseVideo {
    pub title: String,
    pub duration: String,
    pub views: Option<i32>,
}

#[derive(Debug, sqlx::FromRow, Deserialize, Serialize)]
pub struct VideoDto {
    pub title: String,
    pub youtube_id: String,
    pub duration: String,
    pub views: Option<i32>,
}

#[derive(Debug, sqlx::FromRow, Deserialize, Serialize)]
pub struct UpdateVideoDto {
    pub title: Option<String>,
    pub youtube_id: Option<String>,
    pub duration: Option<String>,
    pub views: Option<i32>,
}

#[derive(Debug, sqlx::FromRow, Deserialize, Serialize)]
pub struct CategoryDto {
    pub category_id: String,
}

#[derive(Debug, sqlx::FromRow, Deserialize, Serialize)]
pub struct CategoryName {
    pub name: String,
}

#[derive(Debug, sqlx::FromRow, Deserialize, Serialize)]
pub struct CreateCategory {
    pub name: String,
    pub id: String,
}
