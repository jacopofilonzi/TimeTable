mod api;
mod cache;
mod config;
mod errors;
mod ics;
mod models;
mod shortlinks;
mod startup;
mod state;
mod static_files;
mod universities;

use std::sync::Arc;

use crate::{cache::Cache, config::Config, state::AppState, universities::Registry};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let env_file = startup::load_env_file();
    let config = Config::from_env();

    if std::env::args().nth(1).as_deref() == Some("healthcheck") {
        std::process::exit(startup::run_healthcheck(&config).await);
    }

    startup::init_logging(config.log_format);
    tracing::info!(
        "TimeTable v{} — github.com/jacopofilonzi/TimeTable",
        env!("CARGO_PKG_VERSION")
    );
    env_file.log();

    let static_files = startup::load_frontend(&config)?;
    let short_links = match startup::open_short_links(&config) {
        Ok(links) => links,
        Err(err) => {
            tracing::error!("{err}");
            std::process::exit(1);
        }
    };
    let state = Arc::new(AppState {
        cache: Cache::new(config.redis_url.as_deref()),
        registry: Registry::new(),
        http: startup::http_client()?,
        short_links,
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
