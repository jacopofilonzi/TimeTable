//! Bearer-token authentication for the admin endpoints (`AUTH_TOKEN`).

use std::time::Duration;

use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};

use super::SharedState;
use crate::errors::{AppError, NotFound, Unauthorized};

/// Delay before answering a failed attempt, to slow down token guessing.
const FAILURE_DELAY: Duration = Duration::from_millis(500);

/// Extractor that only succeeds with `Authorization: Bearer <AUTH_TOKEN>`.
///
/// When `AUTH_TOKEN` is not configured the admin endpoints don't exist: the extractor answers
/// 404, like any unknown endpoint.
pub struct AdminAuth;

impl FromRequestParts<SharedState> for AdminAuth {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &SharedState,
    ) -> Result<Self, Self::Rejection> {
        let Some(expected) = &state.config.auth_token else {
            return Err(NotFound::new("Unknown API endpoint").into());
        };
        let provided = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(str::trim);

        match provided {
            Some(token) if constant_time_eq(token.as_bytes(), expected.expose().as_bytes()) => {
                Ok(Self)
            }
            _ => {
                tracing::warn!(path = %parts.uri.path(), "rejected admin request: invalid token");
                tokio::time::sleep(FAILURE_DELAY).await;
                Err(Unauthorized::new("Missing or invalid token").into())
            }
        }
    }
}

/// Compares without short-circuiting on the first different byte (only the length can leak).
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

#[cfg(test)]
mod tests {
    use super::constant_time_eq;

    #[test]
    fn compares_tokens() {
        assert!(constant_time_eq(b"secret", b"secret"));
        assert!(!constant_time_eq(b"secret", b"secreT"));
        assert!(!constant_time_eq(b"secret", b"secret2"));
        assert!(!constant_time_eq(b"", b"x"));
    }
}
