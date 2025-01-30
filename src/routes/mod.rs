use axum::Router;
use tower_http::trace::TraceLayer;

use crate::handlers::auth::auth_handler;

pub mod auth;

pub fn create_routes() -> Router {
    Router::new()
        .nest("/auth", auth_handler())
        .layer(TraceLayer::new_for_http())
}
