use std::sync::Arc;

use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Extension, Json, Router,
};

use crate::{
    models::news_post::{CreateNewsPostDto, PostCommentDto, UpdateNewsPost, UpdatePostCommentDto},
    AppState, Result,
};

pub fn news_posts_handler() -> Router {
    Router::new()
        .route("/get-posts", get(get_posts))
        .route(
            "/get-all-posts-with-comments",
            get(get_all_posts_with_comments),
        )
        .route(
            "/get-posts-with-comments/{id}",
            get(get_posts_with_comments),
        )
        .route("/create-post/{id}", post(create_post))
        .route("/update-post/{id}", put(update_post))
        .route("/delete-post/{id}", delete(delete_post))
        .route("/comments", get(comments))
        .route("/create-comment/{id}", post(create_comment))
        .route("/update-comment/{id}", put(update_comment))
        .route("/delete-comment/{id}", delete(delete_comment))
}

async fn get_posts(Extension(app_state): Extension<Arc<AppState>>) -> Result<impl IntoResponse> {
    let posts = app_state.news_post_service.get_news_posts().await?;
    Ok((StatusCode::OK, Json(posts)))
}

async fn get_all_posts_with_comments(
    Extension(app_state): Extension<Arc<AppState>>,
) -> Result<impl IntoResponse> {
    let posts = app_state
        .news_post_service
        .get_all_posts_with_comments()
        .await?;

    Ok((StatusCode::OK, Json(posts)))
}

async fn get_posts_with_comments(
    Extension(app_state): Extension<Arc<AppState>>,
    Path(post_id): Path<String>,
) -> Result<impl IntoResponse> {
    let posts = app_state
        .news_post_service
        .get_posts_with_comments(&post_id)
        .await?;

    Ok((StatusCode::OK, Json(posts)))
}

async fn create_post(
    Extension(app_state): Extension<Arc<AppState>>,
    Path(author_id): Path<String>,
    Json(news_post): Json<CreateNewsPostDto>,
) -> Result<impl IntoResponse> {
    let new_post = app_state
        .news_post_service
        .create_news_post(news_post, &author_id)
        .await?;
    Ok((StatusCode::CREATED, Json(new_post)))
}

async fn update_post(
    Extension(app_state): Extension<Arc<AppState>>,
    Path(news_post_id): Path<String>,
    Json(update_news_post): Json<UpdateNewsPost>,
) -> Result<impl IntoResponse> {
    let upadeted_post = app_state
        .news_post_service
        .update_news_post(
            &news_post_id,
            update_news_post.url.as_deref(),
            update_news_post.description.as_deref(),
        )
        .await?;

    Ok((StatusCode::CREATED, Json(upadeted_post)))
}

async fn delete_post(
    Extension(app_state): Extension<Arc<AppState>>,
    Path(post_id): Path<String>,
) -> Result<impl IntoResponse> {
    app_state
        .news_post_service
        .delete_news_post(&post_id)
        .await?;

    Ok((StatusCode::NO_CONTENT, "successes"))
}

async fn comments(
    Extension(app_state): Extension<Arc<AppState>>,
    Path(post_id): Path<String>,
) -> Result<impl IntoResponse> {
    let comments = app_state
        .news_post_service
        .get_comments_for_post(&post_id)
        .await?;

    Ok((StatusCode::OK, Json(comments)))
}

async fn create_comment(
    Extension(app_state): Extension<Arc<AppState>>,
    Path(author_id): Path<String>,
    Json(news_comment_post): Json<PostCommentDto>,
) -> Result<impl IntoResponse> {
    let comments = app_state
        .news_post_service
        .create_comment(
            &news_comment_post.id,
            &news_comment_post.content,
            &author_id,
            &news_comment_post.author_name,
        )
        .await?;

    Ok((StatusCode::OK, Json(comments)))
}

async fn update_comment(
    Extension(app_state): Extension<Arc<AppState>>,
    Path(comment_id): Path<String>,
    Json(news_comment_post): Json<UpdatePostCommentDto>,
) -> Result<impl IntoResponse> {
    let comments = app_state
        .news_post_service
        .update_comment(&comment_id, news_comment_post.content.as_deref())
        .await?;

    Ok((StatusCode::OK, Json(comments)))
}

async fn delete_comment(
    Extension(app_state): Extension<Arc<AppState>>,
    Path(comment_id): Path<String>,
) -> Result<()> {
    app_state
        .news_post_service
        .delete_comment(&comment_id)
        .await?;
    Ok(())
}