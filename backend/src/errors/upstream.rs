use std::fmt::Display;

use axum::http::StatusCode;

use super::ApiError;

/// A university's website failed or returned something unexpected.
#[derive(Debug, Clone, thiserror::Error)]
#[error("upstream error: {0}")]
pub struct Upstream(String);

impl Upstream {
    /// `context` describes the failed operation, e.g. `"unicam lessons request"`.
    pub fn new(context: &str, err: impl Display) -> Self {
        Self(format!("{context}: {err}"))
    }

    pub fn msg(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl ApiError for Upstream {
    const STATUS: StatusCode = StatusCode::BAD_GATEWAY;
    const CODE: &'static str = "upstream_error";

    fn public_message(&self) -> String {
        "The university's service is unavailable or returned invalid data".into()
    }
}
