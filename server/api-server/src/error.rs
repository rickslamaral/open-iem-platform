//! Error types for api-server.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

/// Application error returned from handlers.
#[derive(Debug, Error)]
pub enum ApiError {
    /// Authentication failed (bad credentials or expired token).
    #[error("unauthorized: {0}")]
    Unauthorized(&'static str),

    /// Resource not found.
    #[error("not found: {0}")]
    NotFound(String),

    /// Bad request payload.
    #[error("bad request: {0}")]
    BadRequest(String),

    /// Internal server error.
    #[error("internal error: {0}")]
    Internal(String),

    /// RBAC: insufficient permissions.
    #[error("forbidden: {0}")]
    Forbidden(&'static str),

    /// Too many failed authentication attempts.
    #[error("too many requests")]
    TooManyRequests,
}

#[derive(Serialize)]
struct ErrorBody {
    code: String,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            ApiError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED"),
            ApiError::NotFound(_) => (StatusCode::NOT_FOUND, "NOT_FOUND"),
            ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, "BAD_REQUEST"),
            ApiError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"),
            ApiError::Forbidden(_) => (StatusCode::FORBIDDEN, "FORBIDDEN"),
            ApiError::TooManyRequests => (StatusCode::TOO_MANY_REQUESTS, "TOO_MANY_REQUESTS"),
        };
        let message = match &self {
            ApiError::Internal(_) => "internal server error".to_owned(),
            _ => self.to_string(),
        };
        let body = Json(ErrorBody {
            code: code.to_owned(),
            message,
        });
        let mut response = (status, body).into_response();
        if matches!(&self, ApiError::TooManyRequests) {
            response.headers_mut().insert(
                axum::http::header::RETRY_AFTER,
                axum::http::HeaderValue::from_static("5"),
            );
        }
        response
    }
}
