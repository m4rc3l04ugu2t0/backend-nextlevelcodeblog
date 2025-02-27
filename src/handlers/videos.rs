use std::sync::Arc;

use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Extension, Json, Router,
};

use crate::{
    models::query::{CategoryDto, CategoryName, UpdateVideoDto, VideoDto},
    AppState, Result,
};

pub fn videos_handler() -> Router {
    Router::new()
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
}

async fn create_video(
    Extension(app_state): Extension<Arc<AppState>>,
    Json(create_video): Json<VideoDto>,
) -> Result<impl IntoResponse> {
    app_state
        .videos_service
        .create_video(
            &create_video.title,
            &create_video.youtube_id,
            &create_video.duration,
            create_video.views,
        )
        .await?;

    Ok(StatusCode::CREATED)
}

async fn get_video_by_youtube_id(
    Extension(app_state): Extension<Arc<AppState>>,
    Path(youtube_id): Path<String>,
) -> Result<impl IntoResponse> {
    let video = app_state
        .videos_service
        .get_video_by_youtube_id(&youtube_id)
        .await?;
    Ok((StatusCode::OK, Json(video)))
}

async fn get_videos(Extension(app_state): Extension<Arc<AppState>>) -> Result<impl IntoResponse> {
    let videos = app_state.videos_service.videos().await?;

    Ok((StatusCode::OK, Json(videos)))
}

async fn remove_category_from_video(
    Extension(app_state): Extension<Arc<AppState>>,
    Path(video_id): Path<String>,
    Json(category): Json<CategoryDto>,
) -> Result<()> {
    app_state
        .videos_service
        .remove_category_from_video(&video_id, &category.category_id)
        .await?;

    Ok(())
}

async fn add_category_to_video(
    Extension(app_state): Extension<Arc<AppState>>,
    Path(video_id): Path<String>,
    Json(category): Json<CategoryDto>,
) -> Result<impl IntoResponse> {
    app_state
        .videos_service
        .add_category_to_video(&video_id, &category.category_id)
        .await?;

    Ok(StatusCode::OK)
}

async fn update_video(
    Extension(app_state): Extension<Arc<AppState>>,
    Path(video_id): Path<String>,
    Json(update_video): Json<UpdateVideoDto>,
) -> Result<impl IntoResponse> {
    app_state
        .videos_service
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

async fn delete_video(
    Extension(app_state): Extension<Arc<AppState>>,
    Path(video_id): Path<String>,
) -> Result<impl IntoResponse> {
    app_state.videos_service.delete_video(&video_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn create_category(
    Extension(app_state): Extension<Arc<AppState>>,
    Json(category): Json<CategoryName>,
) -> Result<impl IntoResponse> {
app_state
        .videos_service
        .create_category(&category.name)
        .await?;

    Ok(StatusCode::CREATED)
}

async fn delete_category(
    Extension(app_state): Extension<Arc<AppState>>,
    Path(category_id): Path<String>,
) -> Result<impl IntoResponse> {
    app_state
        .videos_service
        .delete_category(&category_id)
        .await?;

    Ok(StatusCode::CREATED)
}
