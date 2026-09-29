//! HTTP layer: routes (`/api/...` + static frontend), `BASE_PATH` mounting, middleware.

mod auth;
mod base_path;
mod handlers;

use std::{collections::HashMap, sync::Arc};

use axum::{
    Router,
    extract::{Path, Query, State},
    http::{HeaderMap, Method, Uri},
    response::IntoResponse,
    routing::{delete, get, post},
};
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

use crate::{
    errors::{AppError, NotFound},
    state::AppState,
    static_files::StaticFiles,
};

pub type SharedState = Arc<AppState>;

pub fn router(state: SharedState, static_files: StaticFiles) -> Router {
    let api = Router::new()
        .route("/health", get(handlers::health))
        .route("/universities", get(handlers::list_universities))
        .route("/universities/{id}", get(handlers::get_university))
        .route(
            "/universities/{id}/options/{field}",
            get(handlers::get_options),
        )
        .route("/universities/{id}/lessons", get(handlers::get_lessons))
        .route(
            "/universities/{id}/lessons.ics",
            get(handlers::get_lessons_ics),
        )
        .route("/short", post(handlers::create_short_link))
        .route("/admin/cache", delete(handlers::clear_cache))
        .fallback(|| async { AppError::from(NotFound::new("Unknown API endpoint")) });

    // Short links, also as `/S/...`: QR codes carry them in uppercase (alphanumeric mode).
    let short_link = {
        let static_files = static_files.clone();
        move |State(state): State<SharedState>,
              Path(code): Path<String>,
              Query(query): Query<HashMap<String, String>>,
              method: Method,
              uri: Uri,
              headers: HeaderMap| {
            let static_files = static_files.clone();
            async move {
                match handlers::open_short_link(&state, &code, &query, &headers).await {
                    Ok(res) => res,
                    // Unknown code or short links disabled: the site's 404 page, not JSON.
                    Err(AppError::NotFound(_)) => static_files.respond(&method, &uri, &headers),
                    Err(err) => err.into_response(),
                }
            }
        }
    };

    let app = Router::new()
        .nest("/api", api)
        .route("/s/{code}", get(short_link.clone()))
        .route("/S/{code}", get(short_link))
        .fallback(move |method: Method, uri: Uri, headers: HeaderMap| {
            let static_files = static_files.clone();
            async move { static_files.respond(&method, &uri, &headers) }
        });

    let base = state.config.base_path.clone();
    let app = app.with_state(state);
    let app = if base.is_empty() {
        app
    } else {
        base_path::mount(base, app)
    };

    app.layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
}
