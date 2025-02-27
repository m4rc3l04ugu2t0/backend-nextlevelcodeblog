use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    body::Body,
    extract::{FromRequestParts, Request},
    http::{header, request::Parts},
    middleware::Next,
    response::IntoResponse,
    Extension,
};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use tower_cookies::Cookies;
use tracing::info;
use uuid::Uuid;

use crate::{
    models::users::{User, UserRole},
    AppState, Error, Result,
};
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JWTAuthMiddeware {
    pub user: User,
}

pub async fn auth(mut req: Request, next: Next) -> Result<impl IntoResponse> {
    info!("ksksk");
    let app_state = req
        .extensions()
        .get::<Arc<AppState>>()
        .ok_or(Error::BadRequest("msmsmsmss1".to_string()))?;

    info!("Checking for token");
    let cookies = CookieJar::from_headers(req.headers());

    let cookie = cookies
        .get("token")
        .map(|c| c.value().to_string())
        .or_else(|| {
            req.headers()
                .get(header::AUTHORIZATION)
                .and_then(|auth_header| auth_header.to_str().ok())
                .and_then(|auth_value| {
                    auth_value
                        .strip_prefix("Bearer ")
                        .map(|stripped| stripped.to_string())
                })
        });

    let token = cookie.ok_or(Error::Unauthorized)?;
    info!("Token found: {}", token);

    let token_details = app_state
        .auth_service
        .decode_token(token)
        .map_err(|_| Error::Unauthorized)?;

    let user_id = Uuid::parse_str(&token_details.to_string()).map_err(|_| Error::Unauthorized)?;
    info!("user_id => {}", user_id);

    let user = app_state
        .auth_service
        .get_user(Some(user_id), None, None, None)
        .await?;

    println!("{:?}", user);

    req.extensions_mut().insert(JWTAuthMiddeware { user });

    Ok(next.run(req).await)
}

pub async fn role_check(
    Extension(_app_state): Extension<Arc<AppState>>,
    req: Request,
    next: Next,
    required_roles: Vec<UserRole>,
) -> Result<impl IntoResponse> {
    // let app_state = req
    // .extensions()
    // .get::<AppState>()
    // .ok_or(Error::BadRequest("msmsmsmss".to_string()))?;

    let user = req
        .extensions()
        .get::<JWTAuthMiddeware>()
        .ok_or_else(|| Error::Unauthorized)?;

    if !required_roles.contains(&user.user.role) {
        return Err(Error::Forbidden);
    }

    Ok(next.run(req).await)
}
