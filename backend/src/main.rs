mod api;
mod cache;
mod config;
mod errors;
mod ics;
mod models;
mod startup;
mod state;
mod static_files;
mod universities;

use std::sync::Arc;

use crate::{cache::Cache, config::Config, state::AppState, universities::Registry};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Looks for `.env` in the current directory and its parents (so the repo root `.env` works
    // when running from `backend/`). Real environment variables take precedence.
    let env_file = dotenvy::dotenv().ok();
    let config = Config::from_env();

    if std::env::args().nth(1).as_deref() == Some("healthcheck") {
        std::process::exit(startup::run_healthcheck(&config).await);
    }

    startup::init_logging(config.log_format);
    tracing::info!(
        "TimeTable v{} — github.com/jacopofilonzi/TimeTable",
        env!("CARGO_PKG_VERSION")
    );
    if let Some(path) = env_file {
        tracing::info!("loaded environment from {}", path.display());
    }

    let static_files = startup::load_frontend(&config)?;
    let state = Arc::new(AppState {
        cache: Cache::new(config.redis_url.as_deref()),
        registry: Registry::new(),
        http: startup::http_client()?,
        config: config.clone(),
    });

    let listener = tokio::net::TcpListener::bind((config.host.as_str(), config.port)).await?;
    tracing::info!(
        "listening on http://{}{}/",
        listener.local_addr()?,
        config.base_path
    );

    axum::serve(listener, api::router(state, static_files))
        .with_graceful_shutdown(startup::shutdown_signal())
        .await?;
    tracing::info!("shut down");
    Ok(())
}
