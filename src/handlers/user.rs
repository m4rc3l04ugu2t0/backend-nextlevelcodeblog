use axum::{routing::get, Router};

pub fn users_handler() -> Router {
    Router::new().route("/test", get(test))
}

async fn test() {
    //
}
