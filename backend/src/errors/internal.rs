use axum::http::StatusCode;

use super::ApiError;

/// Bug or misconfiguration on our side.
#[derive(Debug, Clone, thiserror::Error)]
#[error("internal error: {0}")]
pub struct Internal(String);

impl Internal {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl ApiError for Internal {
    const STATUS: StatusCode = StatusCode::INTERNAL_SERVER_ERROR;
    const CODE: &'static str = "internal_error";

    fn public_message(&self) -> String {
        "An internal error occurred".into()
    }
}
