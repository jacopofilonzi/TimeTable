use std::collections::HashMap;

use axum::{
    Json,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};

use crate::{api::SharedState, errors::AppError};

/// `GET /api/universities/{id}/options/{field}?<deps>` → the field's options.
pub async fn get_options(
    State(state): State<SharedState>,
    Path((id, field)): Path<(String, String)>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Response, AppError> {
    let uni = state.registry.get(&id)?;
    let options = state.options(uni, &field, &query).await?;
    Ok(Json(options.as_slice()).into_response())
}
