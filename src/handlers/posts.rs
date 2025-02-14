use std::sync::Arc;

use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Extension, Json, Router,
};
use serde_json::Value;

use crate::{
    models::posts::{CreatePostDto, Post, UpdatePost},
    AppState, Result,
};

pub fn posts_handler() -> Router {
    Router::new()
        .route("/get_posts", get(get_posts))
        .route("/create-post", post(create_post))
        .route("/update-post/{id}", put(update_post))
        .route("/delete-post/{id}", delete(delete_post))
        .route("/videos", get(get_videos))
        .route("/feed", get(feed))
        .route("/post/{name}", get(get_post))
}

async fn get_posts(Extension(app_state): Extension<Arc<AppState>>) -> Result<impl IntoResponse> {
    let posts = app_state.auth_service.get_posts().await?;
    Ok((StatusCode::OK, Json(posts)))
}

async fn create_post(
    Extension(app_state): Extension<Arc<AppState>>,
    Json(new_post): Json<CreatePostDto>,
) -> Result<impl IntoResponse> {
    let new_post = app_state
        .auth_service
        .create_post(
            &new_post.user_id,
            &new_post.name,
            &new_post.title,
            &new_post.description,
            &new_post.cover_image,
        )
        .await?;
    Ok((StatusCode::CREATED, Json(new_post)))
}

async fn update_post(
    Extension(app_state): Extension<Arc<AppState>>,
    Path(post_id): Path<String>,
    Json(update_post): Json<UpdatePost>,
) -> Result<impl IntoResponse> {
    let upadeted_post = app_state
        .auth_service
        .update_post(
            &post_id,
            update_post.name.as_deref(),
            update_post.title.as_deref(),
            update_post.description.as_deref(),
            update_post.cover_image.as_deref(),
        )
        .await?;

    Ok((StatusCode::CREATED, Json(upadeted_post)))
}

async fn delete_post(
    Extension(app_state): Extension<Arc<AppState>>,
    Path(post_id): Path<String>,
) -> Result<impl IntoResponse> {
    let deleted_post = app_state.auth_service.delete_post(&post_id).await?;

    Ok((StatusCode::GONE, "successes"))
}

async fn get_videos() {}

async fn feed() {}

async fn get_post() {}
