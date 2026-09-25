use axum::http::StatusCode;

use super::ApiError;

/// The client sent missing or invalid parameters.
#[derive(Debug, Clone, thiserror::Error)]
#[error("{0}")]
pub struct BadRequest(String);

impl BadRequest {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl ApiError for BadRequest {
    const STATUS: StatusCode = StatusCode::BAD_REQUEST;
    const CODE: &'static str = "bad_request";
}
