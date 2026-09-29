use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, header},
    response::{IntoResponse, Response},
};
use chrono::Utc;

use super::status_of;
use crate::{
    api::SharedState,
    errors::AppError,
    ics,
    state::{AppState, Endpoint, TrackedRequest},
    universities::University,
};

/// Longest accepted calendar name (`name` query parameter), in characters.
const MAX_NAME_CHARS: usize = 100;

/// `GET /api/universities/{id}/lessons.ics?<params>&weeks=N&name=…&k=…` → iCalendar feed.
pub async fn get_lessons_ics(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Query(query): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let result = match state.registry.get(&id) {
        Ok(uni) => ics_response(&state, uni, &query).await,
        Err(err) => Err(err),
    };
    state.track(TrackedRequest {
        endpoint: Endpoint::LessonsIcs,
        university: Some(&id),
        query: &query,
        headers: &headers,
        status: status_of(&result),
    });
    result
}

/// The feed for `query` (schema fields, `weeks`, optional `name`); shared with short links.
pub(super) async fn ics_response(
    state: &AppState,
    uni: &Arc<dyn University>,
    query: &HashMap<String, String>,
) -> Result<Response, AppError> {
    let result = state.lessons(uni, query).await?;

    let name: String = query
        .get("name")
        .map(|n| n.trim())
        .filter(|n| !n.is_empty())
        .map(|n| n.chars().take(MAX_NAME_CHARS).collect())
        .unwrap_or_else(|| uni.info().name.default_lang().to_string());

    let body = ics::render(&name, uni.timezone().name(), &result.lessons, Utc::now());
    Ok((
        [
            (
                header::CONTENT_TYPE,
                "text/calendar; charset=utf-8".to_string(),
            ),
            (
                header::CONTENT_DISPOSITION,
                format!("inline; filename=\"{}.ics\"", uni.info().id),
            ),
            (header::CACHE_CONTROL, "public, max-age=900".to_string()),
        ],
        body,
    )
        .into_response())
}
