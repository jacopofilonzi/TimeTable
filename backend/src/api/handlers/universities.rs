use axum::{
    Json,
    extract::{Path, State},
    response::{IntoResponse, Response},
};

use crate::{api::SharedState, errors::AppError};

/// `GET /api/universities` → every university with its wizard schema.
pub async fn list_universities(State(state): State<SharedState>) -> Response {
    let infos: Vec<_> = state.registry.all().map(|u| u.info()).collect();
    Json(infos).into_response()
}

/// `GET /api/universities/{id}` → one university with its wizard schema.
pub async fn get_university(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    Ok(Json(state.registry.get(&id)?.info()).into_response())
}
