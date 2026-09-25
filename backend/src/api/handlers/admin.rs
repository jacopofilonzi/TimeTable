use axum::{Json, extract::State};

use crate::{
    api::{SharedState, auth::AdminAuth},
    cache::ClearReport,
};

/// `DELETE /api/admin/cache` (Bearer `AUTH_TOKEN`) → empties the in-memory cache and this app's
/// Redis keys; returns what was removed.
pub async fn clear_cache(_auth: AdminAuth, State(state): State<SharedState>) -> Json<ClearReport> {
    let report = state.cache.clear().await;
    tracing::info!(?report, "cache cleared by admin");
    Json(report)
}
