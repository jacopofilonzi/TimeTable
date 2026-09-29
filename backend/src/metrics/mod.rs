//! Prometheus metrics, exposed by `GET /api/metrics` (Bearer `STATS_TOKEN`) in the OpenMetrics
//! text format.
//!
//! Metrics live in one process-wide registry ([`get`]), so any module can record without
//! plumbing. Counters and histograms are updated where things happen; gauges that describe
//! current state (cache size, Redis status, database sizes, subscriber counts) are refreshed
//! right before each scrape (`AppState::refresh_metrics`). Labels only ever carry bounded values
//! (route templates, university ids, schema params, client families): never track ids, user
//! agents or raw paths.

mod families;
mod http;
mod labels;
mod process;
mod upstream;

use std::sync::LazyLock;

pub use families::Metrics;
pub use http::track_http;
pub use labels::*;
pub use upstream::observe_upstream;

/// `Content-Type` of [`encode`]'s output.
pub const CONTENT_TYPE: &str = "application/openmetrics-text; version=1.0.0; charset=utf-8";

static METRICS: LazyLock<Metrics> = LazyLock::new(Metrics::new);

/// The process-wide metrics.
pub fn get() -> &'static Metrics {
    &METRICS
}

/// Every metric in the OpenMetrics text format.
pub fn encode() -> Result<String, std::fmt::Error> {
    let mut out = String::new();
    prometheus_client::encoding::text::encode(&mut out, &get().registry)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_registered_metrics() {
        get().short_links_created.inc();
        let text = encode().unwrap();
        assert!(text.contains("timetable_short_links_created_total"));
        assert!(text.contains("timetable_build_info"));
        assert!(text.ends_with("# EOF\n"));
    }
}
