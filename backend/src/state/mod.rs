//! Shared application state plus the schema-driven validation and caching around crawlers.

mod lessons;
mod options;
mod validation;

use crate::{cache::Cache, config::Config, universities::Registry};

pub struct AppState {
    pub config: Config,
    pub registry: Registry,
    pub cache: Cache,
    /// Shared HTTP client for all crawlers.
    pub http: reqwest::Client,
}
