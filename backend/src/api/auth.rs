//! Bearer-token authentication: [`AdminAuth`] (`AUTH_TOKEN`, admin actions) and [`StatsAuth`]
//! (`STATS_TOKEN`, read-only metrics and usage log).

use std::time::Duration;

use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};

use super::SharedState;
use crate::{
    config::Secret,
    errors::{AppError, NotFound, Unauthorized},
    metrics::{self, AuthLabels},
};

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
        check(parts, state.config.auth_token.as_ref(), "admin").await?;
        Ok(Self)
    }
}

/// Extractor that only succeeds with `Authorization: Bearer <STATS_TOKEN>`; 404 when
/// `STATS_TOKEN` is not configured.
pub struct StatsAuth;

impl FromRequestParts<SharedState> for StatsAuth {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &SharedState,
    ) -> Result<Self, Self::Rejection> {
        check(parts, state.config.stats_token.as_ref(), "stats").await?;
        Ok(Self)
    }
}

async fn check(
    parts: &Parts,
    expected: Option<&Secret>,
    scope: &'static str,
) -> Result<(), AppError> {
    let Some(expected) = expected else {
        return Err(NotFound::new("Unknown API endpoint").into());
    };
    let provided = parts
        .headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim);

    match provided {
        Some(token) if constant_time_eq(token.as_bytes(), expected.expose().as_bytes()) => Ok(()),
        _ => {
            tracing::warn!(path = %parts.uri.path(), scope, "rejected request: invalid token");
            metrics::get()
                .auth_failures
                .get_or_create(&AuthLabels { scope })
                .inc();
            tokio::time::sleep(FAILURE_DELAY).await;
            Err(Unauthorized::new("Missing or invalid token").into())
        }
    }
}

/// Compares without short-circuiting on the first different byte (only the length can leak).
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;

    #[test]
    fn compares_tokens() {
        assert!(constant_time_eq(b"secret", b"secret"));
        assert!(!constant_time_eq(b"secret", b"secreT"));
        assert!(!constant_time_eq(b"secret", b"secret2"));
        assert!(!constant_time_eq(b"", b"x"));
    }

    fn parts(authorization: Option<&str>) -> Parts {
        let mut req = Request::builder().uri("/api/metrics");
        if let Some(value) = authorization {
            req = req.header(header::AUTHORIZATION, value);
        }
        req.body(()).unwrap().into_parts().0
    }

    #[tokio::test]
    async fn checks_bearer_tokens() {
        let secret = Secret::new("s3cret".into());
        assert!(
            check(&parts(Some("Bearer s3cret")), Some(&secret), "stats")
                .await
                .is_ok()
        );
        let wrong = check(&parts(Some("Bearer nope")), Some(&secret), "stats").await;
        assert!(matches!(wrong, Err(AppError::Unauthorized(_))));
        let missing = check(&parts(None), Some(&secret), "stats").await;
        assert!(matches!(missing, Err(AppError::Unauthorized(_))));
        // No token configured: the endpoint doesn't exist.
        let disabled = check(&parts(Some("Bearer s3cret")), None, "stats").await;
        assert!(matches!(disabled, Err(AppError::NotFound(_))));
    }
}
