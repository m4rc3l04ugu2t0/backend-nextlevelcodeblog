use std::sync::Arc;

use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Extension, Json, Router,
};

use crate::{
    models::{
        news_post::{CreateNewsPostDto, PostCommentDto, UpdateNewsPost, UpdatePostCommentDto},
        query::{CategoryDto, CategoryName,  UpdateVideoDto, VideoDto},
    },
    AppState, Result,
};

pub fn posts_handler() -> Router {
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
        .route("/videos", get(get_videos))
        .route("/create-video", post(create_video))
        .route("/update-video/{id}", put(update_video))
        .route("/delete-video/{id}", delete(delete_video))
        .route("/add-category-video/{id}", post(add_category_to_video))
        .route("/get-video/{id}", get(get_video_by_youtube_id))
        .route(
            "/remove-category-video/{id}",
            delete(remove_category_from_video),
        )
        .route("/create-category", post(create_category))
        .route("/delete-category", post(delete_category))
        .route("/comments", get(comments))
        .route("/create-comment/{id}", post(create_comment))
        .route("/update-comment/{id}", put(update_comment))
        .route("/delete-comment/{id}", delete(delete_comment))
        .route("/post/{name}", get(get_post))
}

async fn get_posts(Extension(app_state): Extension<Arc<AppState>>) -> Result<impl IntoResponse> {
    let posts = app_state.auth_service.get_news_posts().await?;
    Ok((StatusCode::OK, Json(posts)))
}

async fn get_all_posts_with_comments(
    Extension(app_state): Extension<Arc<AppState>>,
) -> Result<impl IntoResponse> {
    let posts = app_state.auth_service.get_all_posts_with_comments().await?;

    Ok((StatusCode::OK, Json(posts)))
}

async fn get_posts_with_comments(
    Extension(app_state): Extension<Arc<AppState>>,
    Path(post_id): Path<String>,
) -> Result<impl IntoResponse> {
    let posts = app_state
        .auth_service
        .get_posts_with_comments(&post_id)
        .await?;

    Ok((StatusCode::OK, Json(posts)))
}

async fn get_video_by_youtube_id(
    Extension(app_state): Extension<Arc<AppState>>,
    Path(youtube_id): Path<String>,
) -> Result<impl IntoResponse> {
    let video = app_state
        .auth_service
        .get_video_by_youtube_id(&youtube_id)
        .await?;
    Ok((StatusCode::OK, Json(video)))
}

async fn update_video(
    Extension(app_state): Extension<Arc<AppState>>,
    Path(video_id): Path<String>,
    Json(update_video): Json<UpdateVideoDto>,
) -> Result<impl IntoResponse> {
   app_state
        .auth_service
        .update_video(
            &video_id,
            update_video.title.as_deref(),
            update_video.youtube_id.as_deref(),
            update_video.duration.as_deref(),
            update_video.views,
        )
        .await?;

    Ok((StatusCode::OK, Json(update_video)))
}

async fn create_category(
    Extension(app_state): Extension<Arc<AppState>>,
    Json(category): Json<CategoryName>,
) -> Result<impl IntoResponse> {
    let category = app_state
        .auth_service
        .create_category(&category.name)
        .await?;

    Ok((StatusCode::CREATED, Json(category)))
}

async fn delete_category(
    Extension(app_state): Extension<Arc<AppState>>,
    Path(category_id): Path<String>,
) -> Result<impl IntoResponse> {
    app_state.auth_service.delete_category(&category_id).await?;

    Ok(StatusCode::CREATED)
}

async fn delete_video(
    Extension(app_state): Extension<Arc<AppState>>,
    Path(video_id): Path<String>,
) -> Result<impl IntoResponse> {
    app_state.auth_service.delete_video(&video_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn add_category_to_video(
    Extension(app_state): Extension<Arc<AppState>>,
    Path(video_id): Path<String>,
    Json(category): Json<CategoryDto>,
) -> Result<impl IntoResponse> {
    app_state
        .auth_service
        .add_category_to_video(&video_id, &category.category_id)
        .await?;

    Ok(StatusCode::OK)
}

async fn remove_category_from_video(
    Extension(app_state): Extension<Arc<AppState>>,
    Path(video_id): Path<String>,
    Json(category): Json<CategoryDto>,
) -> Result<()> {
    app_state
        .auth_service
        .remove_category_from_video(&video_id, &category.category_id)
        .await?;

    Ok(())
}

async fn create_post(
    Extension(app_state): Extension<Arc<AppState>>,
    Path(author_id): Path<String>,
    Json(news_post): Json<CreateNewsPostDto>,
) -> Result<impl IntoResponse> {
    let new_post = app_state
        .auth_service
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
        .auth_service
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
     app_state.auth_service.delete_news_post(&post_id).await?;

    Ok((StatusCode::NO_CONTENT, "successes"))
}

async fn get_videos(Extension(app_state): Extension<Arc<AppState>>) -> Result<impl IntoResponse> {
    let videos = app_state.auth_service.videos().await?;

    Ok((StatusCode::OK, Json(videos)))
}

async fn create_video(
    Extension(app_state): Extension<Arc<AppState>>,
    Json(create_video): Json<VideoDto>,
) -> Result<impl IntoResponse> {
    app_state
        .auth_service
        .create_video(
            &create_video.title,
            &create_video.youtube_id,
            &create_video.duration,
            create_video.views,
        )
        .await?;

    Ok(StatusCode::CREATED)
}

async fn comments(
    Extension(app_state): Extension<Arc<AppState>>,
    Path(post_id): Path<String>,
) -> Result<impl IntoResponse> {
    let comments = app_state
        .auth_service
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
        .auth_service
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
        .auth_service
        .update_comment(&comment_id, news_comment_post.content.as_deref())
        .await?;

    Ok((StatusCode::OK, Json(comments)))
}

async fn delete_comment(
    Extension(app_state): Extension<Arc<AppState>>,
    Path(comment_id): Path<String>,
) -> Result<()> {
    app_state.auth_service.delete_comment(&comment_id).await?;
    Ok(())
}

async fn get_post() {}
