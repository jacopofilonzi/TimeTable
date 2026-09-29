use crate::{config::Config, stats::FeedStats};

/// Opens the usage log database, or `None` when disabled. Fails (and so stops the server) when the
/// configured file can't be created or written.
pub fn open_feed_stats(config: &Config) -> Result<Option<FeedStats>, String> {
    let Some(path) = &config.feed_stats_db else {
        tracing::info!("usage log disabled (FEED_STATS_DB=false)");
        return Ok(None);
    };
    let stats = FeedStats::open(path, config.feed_stats_retention_days).map_err(|e| {
        format!(
            "cannot open the usage log database {} (set FEED_STATS_DB to a writable path, or \
             FEED_STATS_DB=false to disable it): {e}",
            path.display()
        )
    })?;
    match config.feed_stats_retention_days {
        Some(days) => tracing::info!("usage log stored in {} ({days} days)", path.display()),
        None => tracing::info!("usage log stored in {} (kept forever)", path.display()),
    }
    Ok(Some(stats))
}
