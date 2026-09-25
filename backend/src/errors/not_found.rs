use axum::http::StatusCode;

use super::ApiError;

/// The client asked for something that doesn't exist (unknown university, field, endpoint).
#[derive(Debug, Clone, thiserror::Error)]
#[error("{0}")]
pub struct NotFound(String);

impl NotFound {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl ApiError for NotFound {
    const STATUS: StatusCode = StatusCode::NOT_FOUND;
    const CODE: &'static str = "not_found";
}
