use std::collections::HashMap;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};

use super::status_of;
use crate::{
    api::SharedState,
    errors::AppError,
    state::{Endpoint, TrackedRequest},
};

/// `GET /api/universities` → every university with its wizard schema.
pub async fn list_universities(
    State(state): State<SharedState>,
    Query(query): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Response {
    let infos: Vec<_> = state.registry.all().map(|u| u.info()).collect();
    state.track(TrackedRequest {
        endpoint: Endpoint::Universities,
        university: None,
        query: &query,
        headers: &headers,
        status: StatusCode::OK,
    });
    Json(infos).into_response()
}

/// `GET /api/universities/{id}` → one university with its wizard schema.
pub async fn get_university(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Query(query): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let result = state
        .registry
        .get(&id)
        .map(|uni| Json(uni.info()).into_response());
    state.track(TrackedRequest {
        endpoint: Endpoint::University,
        university: Some(&id),
        query: &query,
        headers: &headers,
        status: status_of(&result),
    });
    result
}
