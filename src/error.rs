use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Database(anyhow::Error),
    DatabaseError(String),
    NotFound(String),
    BadRequest(String),
    Internal(String),
    Validation(String),
    SerializationError(String),
    DeserializationError(String),
    TaoError(String),
    ShardError(String),
    TimeoutError(String),
    ConfigurationError(String),
    IdGenerationError(String),
    // Security and HTTP errors
    Unauthorized(String),
    Forbidden(String),
    TooManyRequests(String),
    ServiceUnavailable(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Database(err) => write!(f, "Database error: {}", err),
            AppError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            AppError::Internal(msg) => write!(f, "Internal error: {}", msg),
            AppError::Validation(msg) => write!(f, "Validation error: {}", msg),
            AppError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            AppError::DeserializationError(msg) => write!(f, "Deserialization error: {}", msg),
            AppError::TaoError(msg) => write!(f, "TAO error: {}", msg),
            AppError::ShardError(msg) => write!(f, "Shard error: {}", msg),
            AppError::TimeoutError(msg) => write!(f, "Timeout error: {}", msg),
            AppError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            AppError::IdGenerationError(msg) => write!(f, "ID generation error: {}", msg),
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AppError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            AppError::TooManyRequests(msg) => write!(f, "Too many requests: {}", msg),
            AppError::ServiceUnavailable(msg) => write!(f, "Service unavailable: {}", msg),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            AppError::Database(err) => {
                tracing::error!("Database error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            AppError::DatabaseError(msg) => {
                tracing::error!("Database error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::SerializationError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            AppError::DeserializationError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            AppError::TaoError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            AppError::ShardError(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg.clone()),
            AppError::TimeoutError(msg) => (StatusCode::REQUEST_TIMEOUT, msg.clone()),
            AppError::ConfigurationError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            AppError::IdGenerationError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            AppError::TooManyRequests(msg) => (StatusCode::TOO_MANY_REQUESTS, msg.clone()),
            AppError::ServiceUnavailable(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg.clone()),
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Database(err)
    }
}

pub type AppResult<T> = Result<T, AppError>;