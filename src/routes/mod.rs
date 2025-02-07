use std::sync::Arc;

use axum::{middleware, Extension, Router};
use tower_http::{services::ServeDir, trace::TraceLayer};
fn routes_static() -> Router {
    Router::new().nest_service("/images", ServeDir::new("/assets"))
}

use crate::{
    handlers::{auth::auth_handler, posts::posts_handler, user::users_handler},
    middleware::auth,
    AppState,
};

pub fn create_routes(app_state: Arc<AppState>) -> Router {
    let api_route = Router::new()
        .nest("/auth", auth_handler())
        .nest("/users", users_handler().layer(middleware::from_fn(auth)))
        .nest("/posts", posts_handler())
        .merge(routes_static())
        .layer(TraceLayer::new_for_http())
        .layer(Extension(app_state));

    Router::new().nest("/api", api_route)
}
