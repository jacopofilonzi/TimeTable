//! HTTP request metrics middleware.

use std::time::Instant;

use axum::{
    extract::{MatchedPath, Request},
    middleware::Next,
    response::Response,
};

use super::{HttpLabels, RouteLabels, get};

/// Counts requests and observes their duration, labelled by route template (never the raw path,
/// which would create a series per URL).
pub async fn track_http(req: Request, next: Next) -> Response {
    let route = match req.extensions().get::<MatchedPath>() {
        Some(path) => path.as_str().to_string(),
        // The fallbacks: unknown API endpoints, or the static frontend (and its 404 page).
        None if req.uri().path().starts_with("/api/") => "unmatched".to_string(),
        None => "static".to_string(),
    };
    let method = req.method().as_str().to_string();
    let start = Instant::now();
    let response = next.run(req).await;

    let metrics = get();
    metrics
        .http_duration
        .get_or_create(&RouteLabels {
            route: route.clone(),
            method: method.clone(),
        })
        .observe(start.elapsed().as_secs_f64());
    metrics
        .http_requests
        .get_or_create(&HttpLabels {
            route,
            method,
            status: response.status().as_u16().to_string(),
        })
        .inc();
    response
}
