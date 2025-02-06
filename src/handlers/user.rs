use std::sync::Arc;

use axum::{
    middleware,
    response::IntoResponse,
    routing::{get, put},
    Extension, Json, Router,
};

use crate::{
    middleware::{role_check, JWTAuthMiddeware},
    models::users::{FilterUserDto, UserData, UserResponseDto, UserRole},
    AppState, Result,
};

pub fn users_handler() -> Router {
    Router::new()
        .route(
            "/me",
            get(get_me).layer(middleware::from_fn(|state, req, next| {
                role_check(state, req, next, vec![UserRole::Admin, UserRole::User])
            })),
        )
        .route(
            "/users",
            get(get_users).layer(middleware::from_fn(|state, req, next| {
                role_check(state, req, next, vec![UserRole::Admin])
            })),
        )
        .route("/name", put(update_user_name))
        .route("/role", put(update_user_role))
        .route("/password", put(update_user_password))
}

async fn get_me(
    Extension(_app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddeware>,
) -> Result<impl IntoResponse> {
    let filtered_user = FilterUserDto::filter_user(&user.user);

    let response_data = UserResponseDto {
        status: "success".to_string(),
        data: UserData {
            user: filtered_user,
        },
    };

    Ok(Json(response_data))
}

async fn get_users() {
    // Get all users
}

async fn update_user_name() {
    // Update the user's name
}

async fn update_user_role() {
    // Update the user's role
}

async fn update_user_password() {
    // Update the user's password
}
