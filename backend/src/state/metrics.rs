//! Gauges describing the current state, refreshed right before each Prometheus scrape.

use std::{ffi::OsString, path::Path};

use super::AppState;
use crate::metrics::{
    self, DbLabels, ParamsLabels, RedisStatusLabels, UniversityLabels, WindowLabels,
};

impl AppState {
    pub async fn refresh_metrics(&self) {
        let m = metrics::get();
        m.cache_entries
            .set(i64::try_from(self.cache.entry_count()).unwrap_or(i64::MAX));

        let redis = match (&self.config.redis_url, self.cache.redis_connected()) {
            (None, _) => "disabled",
            (Some(_), true) => "connected",
            (Some(_), false) => "disconnected",
        };
        for status in ["disabled", "connected", "disconnected"] {
            m.redis_status
                .get_or_create(&RedisStatusLabels { status })
                .set(i64::from(status == redis));
        }

        let dbs = [
            ("shortlinks", &self.config.shortlink_db),
            ("feed_stats", &self.config.feed_stats_db),
        ];
        for (db, path) in dbs {
            if let Some(path) = path {
                m.sqlite_file_bytes
                    .get_or_create(&DbLabels { db })
                    .set(sqlite_bytes(path));
            }
        }

        if let Some(links) = &self.short_links {
            match links.count().await {
                Ok(count) => {
                    m.short_links.set(count);
                }
                Err(err) => tracing::warn!(%err, "could not count short links"),
            }
        }

        if let Some(stats) = &self.feed_stats {
            match stats.summary().await {
                Ok(summary) => {
                    // Rebuilt from scratch so that labels which disappeared don't linger.
                    m.feed_subscribers.clear();
                    for (university, count) in summary.subscribers {
                        m.feed_subscribers
                            .get_or_create(&UniversityLabels { university })
                            .set(count);
                    }
                    m.feed_subscribers_active.clear();
                    for (university, window, count) in summary.active {
                        m.feed_subscribers_active
                            .get_or_create(&WindowLabels { university, window })
                            .set(count);
                    }
                    m.feed_anonymous_estimate.clear();
                    for (university, window, count) in summary.anonymous {
                        m.feed_anonymous_estimate
                            .get_or_create(&WindowLabels { university, window })
                            .set(count);
                    }
                    m.feed_subscribers_by_params.clear();
                    for (university, params, count) in summary.by_params {
                        m.feed_subscribers_by_params
                            .get_or_create(&ParamsLabels { university, params })
                            .set(count);
                    }
                }
                Err(err) => tracing::warn!(%err, "could not summarize the usage log"),
            }
        }
    }
}

/// Size of a SQLite database plus its WAL file, 0 if missing.
fn sqlite_bytes(path: &Path) -> i64 {
    let mut wal = OsString::from(path.as_os_str());
    wal.push("-wal");
    let size = |p: &Path| std::fs::metadata(p).map(|m| m.len()).unwrap_or(0);
    i64::try_from(size(path) + size(Path::new(&wal))).unwrap_or(i64::MAX)
}
