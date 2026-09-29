use std::collections::HashMap;

use axum::{
    Json,
    extract::{Query, State},
    http::header,
    response::{IntoResponse, Response},
};

use crate::{
    api::{SharedState, auth::StatsAuth},
    errors::{AppError, BadRequest, Internal, NotFound},
    metrics,
    stats::{FeedStats, Kind, RequestFilter, RequestRow, SubscriberRow},
};

/// Rows returned when `limit` is not given, and the most that can be asked for.
const DEFAULT_LIMIT: u32 = 500;
const MAX_LIMIT: u32 = 10_000;

/// `GET /api/metrics` (Bearer `STATS_TOKEN`) → every metric, OpenMetrics text format.
pub async fn metrics(
    _auth: StatsAuth,
    State(state): State<SharedState>,
) -> Result<Response, AppError> {
    state.refresh_metrics().await;
    let body = metrics::encode().map_err(|e| Internal::new(format!("encoding metrics: {e}")))?;
    Ok(([(header::CONTENT_TYPE, metrics::CONTENT_TYPE)], body).into_response())
}

/// `GET /api/stats/requests?kind=&from=&to=&track_id=&anonymous=&limit=` (Bearer `STATS_TOKEN`)
/// → usage log rows, newest first; `track_id` is `null` for anonymous requests.
pub async fn stats_requests(
    _auth: StatsAuth,
    State(state): State<SharedState>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<Vec<RequestRow>>, AppError> {
    let filter = RequestFilter {
        from: param(&query, "from", parse_time)?,
        to: param(&query, "to", parse_time)?,
        // `all` (e.g. a Grafana "All" variable) or empty means no filter.
        kind: match query.get("kind").map(String::as_str) {
            None | Some("" | "all") => None,
            Some(kind) => Some(
                Kind::parse(kind)
                    .ok_or_else(|| BadRequest::new("Invalid 'kind': use feed, api or all"))?
                    .as_str(),
            ),
        },
        track_id: query.get("track_id").filter(|t| !t.is_empty()).cloned(),
        anonymous: param(&query, "anonymous", |v| match v {
            "true" => Some(Some(true)),
            "false" => Some(Some(false)),
            "any" | "all" => Some(None),
            _ => None,
        })?
        .flatten(),
        limit: limit(&query)?,
    };
    Ok(Json(feed_stats(&state)?.requests(filter).await?))
}

/// `GET /api/stats/feed-subscribers?active_days=&limit=` (Bearer `STATS_TOKEN`) → track ids,
/// most recently seen first.
pub async fn stats_subscribers(
    _auth: StatsAuth,
    State(state): State<SharedState>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<Vec<SubscriberRow>>, AppError> {
    let active_days = param(&query, "active_days", |v| v.parse::<u32>().ok())?;
    Ok(Json(
        feed_stats(&state)?
            .subscribers(active_days, limit(&query)?)
            .await?,
    ))
}

fn feed_stats(state: &SharedState) -> Result<&FeedStats, AppError> {
    state
        .feed_stats
        .as_ref()
        .ok_or_else(|| NotFound::new("The usage log is disabled").into())
}

/// An optional parameter; present but unparsable is a 400.
fn param<T>(
    query: &HashMap<String, String>,
    key: &str,
    parse: impl Fn(&str) -> Option<T>,
) -> Result<Option<T>, AppError> {
    match query.get(key).filter(|v| !v.is_empty()) {
        None => Ok(None),
        Some(v) => parse(v)
            .map(Some)
            .ok_or_else(|| BadRequest::new(format!("Invalid '{key}'")).into()),
    }
}

fn limit(query: &HashMap<String, String>) -> Result<u32, AppError> {
    Ok(param(query, "limit", |v| v.parse::<u32>().ok())?
        .unwrap_or(DEFAULT_LIMIT)
        .clamp(1, MAX_LIMIT))
}

/// Unix seconds or milliseconds (as Grafana's `${__from}`), or RFC 3339.
fn parse_time(value: &str) -> Option<i64> {
    match value.parse::<i64>() {
        Ok(n) if n.abs() >= 100_000_000_000 => Some(n / 1000),
        Ok(n) => Some(n),
        Err(_) => chrono::DateTime::parse_from_rfc3339(value)
            .ok()
            .map(|t| t.timestamp()),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_time;

    #[test]
    fn parses_times() {
        assert_eq!(parse_time("1790000000"), Some(1_790_000_000));
        assert_eq!(parse_time("1790000000123"), Some(1_790_000_000));
        assert_eq!(parse_time("2026-09-29T08:00:00Z"), Some(1_790_668_800));
        assert_eq!(parse_time("yesterday"), None);
    }
}
