use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};
use tracing::error;
use validator::ValidationErrors;

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
    Validation(ValidationErrors),
    ReadString(String),
}

#[derive(Serialize)]
pub struct ValidationResponse {
    pub code: u16,
    pub message: String,
    pub errors: Option<HashMap<String, Vec<String>>>,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Self::NotFound => (StatusCode::NOT_FOUND, "Resource not found").into_response(),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
            Self::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
            }
            Self::BadRequest(msg) => {
                let response = ValidationResponse {
                    message: msg,
                    code: 400,
                    errors: None,
                };

                (StatusCode::BAD_REQUEST, Json(response)).into_response()
            }
            Self::DatabaseError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
            }
            Self::InvalidHashFormat(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Invalid hash format").into_response()
            }
            Self::Forbidden => (StatusCode::FORBIDDEN, "Forbidden").into_response(),
            Self::Validation(errors) => {
                let mut errror_map = HashMap::new();

                for (field, errors) in errors.field_errors() {
                    let messages = errors
                        .iter()
                        .map(|e| e.message.clone().unwrap_or_default().to_string())
                        .collect();
                    errror_map.insert(field.to_string(), messages);
                }
                let response = ValidationResponse {
                    code: StatusCode::BAD_REQUEST.as_u16(),
                    message: "Validation failed".to_string(),
                    errors: Some(errror_map),
                };

                (StatusCode::BAD_REQUEST, Json(response)).into_response()
            }
            Self::ReadString(err) => (StatusCode::FAILED_DEPENDENCY, err).into_response(),
        }
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

impl From<ValidationErrors> for Error {
    fn from(err: ValidationErrors) -> Self {
        error!("Invalid hash format");
        Self::Validation(err)
    }
}

impl From<std::string::String> for Error {
    fn from(err: std::string::String) -> Self {
        error!("Invalid hash format");
        Self::ReadString(err)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub status: String,
    pub message: String,
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}
