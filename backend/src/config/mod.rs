//! Runtime configuration, read once at startup from the environment (and `.env`).
//! Every variable is documented in `.env.example`.

mod base_path;
mod db_path;
mod env;
mod log_format;
mod secret;
mod static_dir;

use std::{path::PathBuf, time::Duration};

pub use log_format::LogFormat;
pub use secret::Secret;

/// `FEED_STATS_RETENTION_DAYS` when unset.
const DEFAULT_RETENTION_DAYS: u32 = 365;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    /// Public path prefix, normalized to either `""` or `"/something"` (no trailing slash).
    pub base_path: String,
    /// Directory containing the built frontend, if any was found.
    pub static_dir: Option<PathBuf>,
    pub redis_url: Option<String>,
    pub lessons_ttl: Duration,
    pub options_ttl: Duration,
    pub log_format: LogFormat,
    /// Token for the admin endpoints (`/api/admin/...`); they are disabled when unset.
    pub auth_token: Option<Secret>,
    /// SQLite file for short links; `None` when disabled (`SHORTLINK_DB=false`).
    pub shortlink_db: Option<PathBuf>,
    /// SQLite file for the usage log (`.ics` and external API requests); `None` when disabled
    /// (`FEED_STATS_DB=false`).
    pub feed_stats_db: Option<PathBuf>,
    /// Days of request log to keep; `None` keeps it forever (`FEED_STATS_RETENTION_DAYS=0`).
    pub feed_stats_retention_days: Option<u32>,
    /// Read-only token for `/api/metrics` and `/api/stats/...`; they are disabled when unset.
    pub stats_token: Option<Secret>,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            host: env::var("HOST").unwrap_or_else(|| "0.0.0.0".into()),
            port: env::parse("PORT").unwrap_or(8080),
            base_path: base_path::normalize(&env::var("BASE_PATH").unwrap_or_default()),
            static_dir: static_dir::resolve(),
            redis_url: env::var("REDIS_URL"),
            lessons_ttl: Duration::from_secs(env::parse("CACHE_LESSONS_TTL").unwrap_or(6 * 3600)),
            options_ttl: Duration::from_secs(env::parse("CACHE_OPTIONS_TTL").unwrap_or(24 * 3600)),
            log_format: LogFormat::from_env_value(env::var("LOG_FORMAT").as_deref()),
            auth_token: env::var("AUTH_TOKEN").map(Secret::new),
            shortlink_db: db_path::resolve("SHORTLINK_DB", db_path::SHORTLINK_DEFAULT),
            feed_stats_db: db_path::resolve("FEED_STATS_DB", db_path::FEED_STATS_DEFAULT),
            feed_stats_retention_days: Some(
                env::parse("FEED_STATS_RETENTION_DAYS").unwrap_or(DEFAULT_RETENTION_DAYS),
            )
            .filter(|&days| days > 0),
            stats_token: env::var("STATS_TOKEN").map(Secret::new),
        }
    }
}
