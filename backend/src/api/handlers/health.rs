use axum::{Json, extract::State};
use serde_json::{Value, json};

use crate::api::SharedState;

/// `GET /api/health` → `{ status, redis: disabled | connected | disconnected }`.
pub async fn health(State(state): State<SharedState>) -> Json<Value> {
    let redis = match (&state.config.redis_url, state.cache.redis_connected()) {
        (None, _) => "disabled",
        (Some(_), true) => "connected",
        (Some(_), false) => "disconnected",
    };
    Json(json!({ "status": "ok", "redis": redis }))
}
