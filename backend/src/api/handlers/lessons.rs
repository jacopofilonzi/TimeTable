use std::collections::HashMap;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use serde::Serialize;

use super::status_of;
use crate::{
    api::SharedState,
    errors::AppError,
    models::Lesson,
    state::{AppState, Endpoint, TrackedRequest},
};

#[derive(Serialize)]
struct LessonsResponse<'a> {
    /// IANA timezone of the university, for displaying times.
    timezone: &'static str,
    from: String,
    to: String,
    lessons: &'a [Lesson],
}

/// `GET /api/universities/{id}/lessons?<params>&weeks=N` → lessons as JSON.
pub async fn get_lessons(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Query(query): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let result = lessons_json(&state, &id, &query).await;
    state.track(TrackedRequest {
        endpoint: Endpoint::Lessons,
        university: Some(&id),
        query: &query,
        headers: &headers,
        status: status_of(&result),
    });
    result
}

async fn lessons_json(
    state: &AppState,
    id: &str,
    query: &HashMap<String, String>,
) -> Result<Response, AppError> {
    let uni = state.registry.get(id)?;
    let result = state.lessons(uni, query).await?;
    Ok(Json(LessonsResponse {
        timezone: uni.timezone().name(),
        from: result.range.from.to_rfc3339(),
        to: result.range.to.to_rfc3339(),
        lessons: &result.lessons,
    })
    .into_response())
}
