use std::collections::HashMap;

use axum::{
    extract::{Path, Query, State},
    http::header,
    response::{IntoResponse, Response},
};
use chrono::Utc;

use crate::{api::SharedState, errors::AppError, ics};

/// Longest accepted calendar name (`name` query parameter), in characters.
const MAX_NAME_CHARS: usize = 100;

/// `GET /api/universities/{id}/lessons.ics?<params>&weeks=N&name=…` → iCalendar feed.
pub async fn get_lessons_ics(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Response, AppError> {
    let uni = state.registry.get(&id)?;
    let result = state.lessons(uni, &query).await?;

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
