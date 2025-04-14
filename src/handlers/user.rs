use std::sync::Arc;

use axum::{
    extract::Path,
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{delete, get, put},
    Extension, Json, Router,
};
use validator::Validate;

use crate::{
    middleware::{role_check, JWTAuthMiddeware},
    models::{
        response::Response,
        users::{
            FilterUserDto, NameUpdateDto, UserData, UserPasswordUpdateDto, UserResponseDto,
            UserRole,
        },
    },
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
        .route("/delete/{id}", delete(delete_user))
        .route("/update-username", put(update_user_name))
        .route("/role", put(update_user_role))
        .route("/update-password", put(update_user_password))
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

async fn delete_user(
    Extension(app_state): Extension<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> Result<impl IntoResponse> {
    app_state.users_service.delete_user(&user_id).await?;

    Ok((StatusCode::NO_CONTENT, "Deleted"))
}

async fn get_users() {
    // Get all users
}

pub async fn update_user_name(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddeware>,
    Json(user_update): Json<NameUpdateDto>,
) -> Result<impl IntoResponse> {
    user_update.validate()?;

    app_state
        .users_service
        .update_username(&user.user, user_update)
        .await?;

    Ok(StatusCode::OK)
}

async fn update_user_role() {
    // Update the user's role
}

pub async fn update_user_password(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddeware>,
    Json(user_update): Json<UserPasswordUpdateDto>,
) -> Result<impl IntoResponse> {
    user_update.validate()?;

    app_state
        .users_service
        .update_user_password(&user.user, user_update)
        .await?;

    let response = Response {
        message: "Password updated Successfully".to_string(),
        status: "success",
    };

    Ok(Json(response))
}
