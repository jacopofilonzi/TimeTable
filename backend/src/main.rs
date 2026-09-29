mod api;
mod cache;
mod config;
mod errors;
mod ics;
mod metrics;
mod models;
mod shortlinks;
mod startup;
mod state;
mod static_files;
mod stats;
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
    let (short_links, feed_stats) = match startup::open_short_links(&config)
        .and_then(|links| Ok((links, startup::open_feed_stats(&config)?)))
    {
        Ok(dbs) => dbs,
        Err(err) => {
            tracing::error!("{err}");
            std::process::exit(1);
        }
    };
    if config.stats_token.is_none() {
        tracing::info!("STATS_TOKEN not set, /api/metrics and /api/stats are disabled");
    }
    // Registers the metrics now, so the process start time is the real one.
    metrics::get();
    let state = Arc::new(AppState {
        cache: Cache::new(config.redis_url.as_deref()),
        registry: Registry::new(),
        http: startup::http_client()?,
        short_links,
        feed_stats,
        config: config.clone(),
    });

    let listener = tokio::net::TcpListener::bind((config.host.as_str(), config.port)).await?;
    tracing::info!(
        "listening on http://{}{}/",
        listener.local_addr()?,
        config.base_path
    );

    axum::serve(listener, api::router(state.clone(), static_files))
        .with_graceful_shutdown(startup::shutdown_signal())
        .await?;
    if let Some(stats) = &state.feed_stats {
        stats.flush().await;
    }
    tracing::info!("shut down");
    Ok(())
}
