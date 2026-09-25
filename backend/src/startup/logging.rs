use tracing_subscriber::{EnvFilter, fmt};

use crate::config::LogFormat;

/// Used when `RUST_LOG` is not set.
const DEFAULT_FILTER: &str = "info,tower_http=info";

pub fn init_logging(format: LogFormat) {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER));
    let builder = fmt().with_env_filter(filter);
    match format {
        LogFormat::Json => builder.json().init(),
        LogFormat::Compact => builder.compact().init(),
        LogFormat::Pretty => builder.init(),
    }
}
