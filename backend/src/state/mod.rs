//! Shared application state plus the schema-driven validation and caching around crawlers.

mod lessons;
mod options;
mod short_links;
mod validation;

use crate::{cache::Cache, config::Config, shortlinks::ShortLinks, universities::Registry};

pub struct AppState {
    pub config: Config,
    pub registry: Registry,
    pub cache: Cache,
    /// Shared HTTP client for all crawlers.
    pub http: reqwest::Client,
    /// `None` when short links are disabled.
    pub short_links: Option<ShortLinks>,
}
