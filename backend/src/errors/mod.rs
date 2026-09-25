//! Application errors. Each error kind lives in its own module and implements [`ApiError`], which
//! defines how it is exposed over HTTP; [`AppError`] wraps them all.

mod bad_request;
mod internal;
mod not_found;
mod unauthorized;
mod upstream;

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

pub use bad_request::BadRequest;
pub use internal::Internal;
pub use not_found::NotFound;
pub use unauthorized::Unauthorized;
pub use upstream::Upstream;

/// How an error kind is exposed over HTTP.
pub trait ApiError: std::error::Error {
    const STATUS: StatusCode;
    /// Machine-readable code, sent as `error` in the JSON body.
    const CODE: &'static str;

    /// Message sent to the client. Server-side errors override it to avoid leaking details.
    fn public_message(&self) -> String {
        self.to_string()
    }
}

/// Any application error. `Clone` so it can be shared between requests coalesced by the cache.
#[derive(Debug, Clone, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    NotFound(#[from] NotFound),
    #[error(transparent)]
    BadRequest(#[from] BadRequest),
    #[error(transparent)]
    Unauthorized(#[from] Unauthorized),
    #[error(transparent)]
    Upstream(#[from] Upstream),
    #[error(transparent)]
    Internal(#[from] Internal),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match &self {
            Self::NotFound(e) => respond(e),
            Self::BadRequest(e) => respond(e),
            Self::Unauthorized(e) => respond(e),
            Self::Upstream(e) => respond(e),
            Self::Internal(e) => respond(e),
        }
    }
}

/// JSON body `{ "error": CODE, "message": ... }`; server-side errors are logged with full details.
fn respond<E: ApiError>(error: &E) -> Response {
    if E::STATUS.is_server_error() {
        tracing::error!(error = %error, code = E::CODE, "request failed");
    }
    (
        E::STATUS,
        Json(json!({ "error": E::CODE, "message": error.public_message() })),
    )
        .into_response()
}
