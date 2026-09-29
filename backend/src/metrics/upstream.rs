//! Instrumentation of the fetches from the university sites.

use std::{future::Future, time::Instant};

use super::{UpstreamLabels, UpstreamResultLabels, get};
use crate::errors::AppError;

/// Runs a crawler call, recording its duration and outcome.
pub async fn observe_upstream<T>(
    university: &'static str,
    operation: &'static str,
    fetch: impl Future<Output = Result<T, AppError>>,
) -> Result<T, AppError> {
    let start = Instant::now();
    let result = fetch.await;
    let metrics = get();
    metrics
        .upstream_duration
        .get_or_create(&UpstreamLabels {
            university,
            operation,
        })
        .observe(start.elapsed().as_secs_f64());
    metrics
        .upstream_requests
        .get_or_create(&UpstreamResultLabels {
            university,
            operation,
            result: if result.is_ok() { "ok" } else { "error" },
        })
        .inc();
    result
}
