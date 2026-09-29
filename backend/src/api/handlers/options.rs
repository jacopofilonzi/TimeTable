use std::collections::HashMap;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
};

use super::status_of;
use crate::{
    api::SharedState,
    errors::AppError,
    state::{Endpoint, TrackedRequest},
};

/// `GET /api/universities/{id}/options/{field}?<deps>` → the field's options.
pub async fn get_options(
    State(state): State<SharedState>,
    Path((id, field)): Path<(String, String)>,
    Query(query): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let result = async {
        let uni = state.registry.get(&id)?;
        let options = state.options(uni, &field, &query).await?;
        Ok(Json(options.as_slice()).into_response())
    }
    .await;
    state.track(TrackedRequest {
        endpoint: Endpoint::Options(&field),
        university: Some(&id),
        query: &query,
        headers: &headers,
        status: status_of(&result),
    });
    result
}
