use std::{env, sync::Arc};

use axum::{
    body::Body,
    extract::Query,
    http::{
        header::{self, ACCEPT, AUTHORIZATION, CONTENT_DISPOSITION, CONTENT_TYPE}, HeaderMap, HeaderName, HeaderValue, Method, Request, StatusCode
    },
    middleware::Next,
    routing::{get, post},
    Extension, Json, Router,
};
use tower_cookies::Cookie;
use tower_http::cors::CorsLayer;
use validator::Validate;

use crate::{
    mail::mails::send_verification_email,
    models::{
        query::VerifyEmailQueryDto,
        response::Response,
        users::{
            ForgotPasswordRequestDto, LoginUserDto, RegisterUserDto, ResetPasswordRequestDto,
            UserLoginResponseDto,
        },
    },
    AppState, Error, Result,
};

use axum::response::IntoResponse;

pub fn auth_handler() -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/verify-email", get(verify_email))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password", post(reset_password))
}

pub async fn register(
    Extension(app_state): Extension<Arc<AppState>>,
    Json(new_user): Json<RegisterUserDto>,
) -> Result<impl IntoResponse> {
    new_user.validate()?;

    let user = app_state
        .auth_service
        .register(new_user.name, new_user.email, new_user.password)
        .await?;

    let token = user
        .verification_token
        .ok_or(Error::BadRequest("Invalid data".to_string()))?;

    send_verification_email(&user.email, &user.name, &token).await?;

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
    Extension(app_state): Extension<Arc<AppState>>,
    Json(user): Json<LoginUserDto>,
) -> Result<impl IntoResponse> {
    user.validate()?;

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
    Extension(app_state): Extension<Arc<AppState>>,
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

pub async fn forgot_password(
    Extension(app_state): Extension<Arc<AppState>>,
    Json(email): Json<ForgotPasswordRequestDto>,
) -> Result<impl IntoResponse> {
    email.validate()?;

    app_state.auth_service.forgot_password(email.email).await?;

    let response = Response {
        message: "Password reset link has been sent to your email.".to_string(),
        status: "success",
    };

    Ok(Json(response))
}

pub async fn reset_password(
    Extension(app_state): Extension<Arc<AppState>>,
    Json(body): Json<ResetPasswordRequestDto>,
) -> Result<impl IntoResponse> {
    body.validate()
        .map_err(|e| Error::BadRequest(e.to_string()))?;

    app_state
        .auth_service
        .reset_password(body.token, body.new_password)
        .await?;

    let response = Response {
        message: "Password has been successfully reset.".to_string(),
        status: "success",
    };

    Ok(Json(response))
}

pub fn configure_cors() -> CorsLayer {
    let x_api_key = HeaderName::from_static("x-api-key");

    CorsLayer::new()
        .allow_origin([env::var("FRONT_URL").expect("FRONT_URL must be set").parse::<HeaderValue>().unwrap()])
        .allow_methods(vec![
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(vec![AUTHORIZATION, CONTENT_TYPE, ACCEPT, x_api_key])
        .allow_credentials(true)
        .expose_headers(vec![CONTENT_DISPOSITION])
        .max_age(std::time::Duration::from_secs(86400))
}

pub async fn require_api_key(
    req: Request<Body>,
    next: Next,
) -> std::result::Result<axum::response::Response, StatusCode> {
    if req.method() == Method::OPTIONS {
        return Ok(next.run(req).await);
    }

    let headers = req.headers();
    let api_key_header = HeaderName::from_static("x-api-key");

    match headers.get(&api_key_header) {
        Some(api_key_value) => {
            let stored_key = env::var("API_KEY").unwrap_or_default();

            if api_key_value.to_str().unwrap_or("") == stored_key {
                Ok(next.run(req).await)
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        None => Err(StatusCode::UNAUTHORIZED),
    }
}
