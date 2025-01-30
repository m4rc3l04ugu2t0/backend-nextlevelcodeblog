use axum::{
    extract::Query,
    http::{header, HeaderMap, StatusCode},
    routing::{get, post},
    Extension, Json, Router,
};
use tower_cookies::Cookie;
use validator::Validate;

use crate::{
    models::{
        query::VerifyEmailQueryDto,
        response::Response,
        users::{LoginUserDto, RegisterUserDto, UserLoginResponseDto},
    },
    AppState, Error, Result,
};

use axum::response::IntoResponse;

pub fn auth_handler() -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/verify", get(verify_email))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password", post(reset_password))
}

pub async fn register(
    Extension(app_state): Extension<AppState>,
    Json(new_user): Json<RegisterUserDto>,
) -> Result<impl IntoResponse> {
    new_user
        .validate()
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    app_state
        .auth_service
        .register(new_user.name, new_user.email, new_user.password)
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(Response {
            status: "success",
            message: "Registration successful! Please check your email to verify your account."
                .to_string(),
        }),
    ))
}

pub async fn login(
    Extension(app_state): Extension<AppState>,
    Json(user): Json<LoginUserDto>,
) -> Result<impl IntoResponse> {
    user.validate()
        .map_err(|e| Error::BadRequest(e.to_string()))?;

    let token = app_state
        .auth_service
        .login(&user.email, &user.password)
        .await?;

    let cookie_duration = time::Duration::minutes(app_state.config.jwt_maxage * 60);
    let cookie = Cookie::build(("token", &token))
        .path("/")
        .max_age(cookie_duration)
        .http_only(true)
        .build();

    let response = Json(UserLoginResponseDto {
        status: "success".to_string(),
        token: token.clone(),
    });

    let mut headers = HeaderMap::new();

    headers.append(header::SET_COOKIE, cookie.to_string().parse().unwrap());

    let mut response = response.into_response();
    response.headers_mut().extend(headers);

    Ok(response)
}

pub async fn verify_email(
    Query(params): Query<VerifyEmailQueryDto>,
    Extension(app_state): Extension<AppState>,
) -> Result<impl IntoResponse> {
    params
        .validate()
        .map_err(|e| Error::BadRequest(e.to_string()))?;

    let token = app_state.auth_service.verify_email(params.token).await?;

    let cookie_duration = time::Duration::minutes(app_state.config.jwt_maxage * 60);
    let cookie = Cookie::build(("token", token.clone()))
        .path("/")
        .max_age(cookie_duration)
        .http_only(true)
        .build();

    let mut headers = HeaderMap::new();

    headers.append(header::SET_COOKIE, cookie.to_string().parse().unwrap());

    let response = Json(Response {
        status: "success",
        message: "Email verified successfully!".to_string(),
    });

    Ok(response)
}

pub async fn forgot_password() {
    // Send a password reset email
}

pub async fn reset_password() {
    // Reset a user's password
}
