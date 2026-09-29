//! Shared application state plus the schema-driven validation and caching around crawlers.

mod lessons;
mod metrics;
mod options;
mod short_links;
mod tracking;
mod validation;

pub use tracking::{Endpoint, TrackedRequest};

use crate::{
    cache::Cache, config::Config, shortlinks::ShortLinks, stats::FeedStats, universities::Registry,
};

pub struct AppState {
    pub config: Config,
    pub registry: Registry,
    pub cache: Cache,
    /// Shared HTTP client for all crawlers.
    pub http: reqwest::Client,
    /// `None` when short links are disabled.
    pub short_links: Option<ShortLinks>,
    /// Usage log; `None` when disabled (`FEED_STATS_DB=false`).
    pub feed_stats: Option<FeedStats>,
}
