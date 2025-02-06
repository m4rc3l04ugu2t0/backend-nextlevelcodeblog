use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tracing::error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    NotFound,
    Unauthorized,
    InternalServerError,
    BadRequest(String),
    DatabaseError(sqlx::Error),
    InvalidHashFormat(argon2::password_hash::Error),
    Forbidden,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::NotFound => (StatusCode::NOT_FOUND, "Resource not found"),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            Self::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            Self::BadRequest(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            Self::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
            Self::InvalidHashFormat(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Invalid hash format")
            }
            Self::Forbidden => (StatusCode::FORBIDDEN, "Forbidden"),
        };

        let body = Json(json!({ "error": message }));
        (status, body).into_response()
    }
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        error!("Database error: {:?}", err);
        Self::DatabaseError(err)
    }
}

impl From<argon2::password_hash::Error> for Error {
    fn from(err: argon2::password_hash::Error) -> Self {
        error!("Invalid hash format");
        Self::InvalidHashFormat(err)
    }
}
