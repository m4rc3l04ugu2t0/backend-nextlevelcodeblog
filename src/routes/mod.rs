use std::{io::Cursor, path::PathBuf, sync::Arc};

use axum::{
    extract::Query,
    http::{HeaderMap, StatusCode},
    middleware,
    response::IntoResponse,
    routing::get,
    Extension, Router,
};
use image::{ImageFormat, ImageReader};
use serde::Deserialize;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::info;

use crate::{
    handlers::{auth::auth_handler, posts::posts_handler, user::users_handler},
    middleware::auth,
    AppState,
};

#[derive(Deserialize)]
struct ImageParams {
    url: String,
    w: Option<u32>,
    q: Option<u8>,
}

async fn handle_image_optimization(
    Query(params): Query<ImageParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // 1. Parse and validate URL
    let decoded_url = urlencoding::decode(&params.url)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid URL encoding".to_string()))?;

    // 2. Extract local file path from URL
    let file_path = PathBuf::from(
        decoded_url
            .strip_prefix("http://localhost:8080/api/images/")
            .ok_or((StatusCode::BAD_REQUEST, "Invalid image URL".to_string()))?,
    );

    // 3. Security check
    if file_path.components().count() != 2 || file_path.is_absolute() {
        return Err((StatusCode::FORBIDDEN, "Invalid file path".to_string()));
    }

    // 4. Load image from filesystem
    let image_bytes = tokio::fs::read(format!("src/assets/{}", file_path.display()))
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, format!("File not found: {}", e)))?;

    // 5. Process image
    let mut img = ImageReader::new(Cursor::new(image_bytes))
        .with_guessed_format()
        .map_err(|e| {
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("Image decode error: {}", e),
            )
        })?
        .decode()
        .map_err(|e| {
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("Image processing error: {}", e),
            )
        })?;

    // 6. Resize if width specified
    if let Some(width) = params.w {
        img = img.resize(
            width,
            (width as f32 * (img.height() as f32 / img.width() as f32)) as u32,
            image::imageops::FilterType::Lanczos3,
        );
    }

    // 7. Convert to WebP
    let mut output_buf = Cursor::new(Vec::new());
    img.write_to(&mut output_buf, ImageFormat::WebP)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Image conversion error: {}", e),
            )
        })?;

    // 8. Set response headers
    let mut headers = HeaderMap::new();
    headers.insert("content-type", "image/webp".parse().unwrap());
    headers.insert(
        "cache-control",
        "public, max-age=31536000, immutable".parse().unwrap(),
    );

    Ok((StatusCode::OK, headers, output_buf.into_inner()))
}

fn routes_static() -> Router {
    Router::new().nest_service("/images", ServeDir::new("src/assets"))
}
pub fn create_routes(app_state: Arc<AppState>) -> Router {
    let api_route = Router::new()
        .nest("/auth", auth_handler())
        .nest("/users", users_handler().layer(middleware::from_fn(auth)))
        .nest("/posts", posts_handler())
        .fallback_service(routes_static())
        .layer(TraceLayer::new_for_http())
        .layer(Extension(app_state));

    Router::new()
        .route("/_next/image", get(handle_image_optimization))
        .nest("/api", api_route)
}
