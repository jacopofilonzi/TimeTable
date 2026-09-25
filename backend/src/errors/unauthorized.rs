use axum::http::StatusCode;

use super::ApiError;

/// Missing or wrong credentials for an admin endpoint.
#[derive(Debug, Clone, thiserror::Error)]
#[error("{0}")]
pub struct Unauthorized(String);

impl Unauthorized {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl ApiError for Unauthorized {
    const STATUS: StatusCode = StatusCode::UNAUTHORIZED;
    const CODE: &'static str = "unauthorized";
}
