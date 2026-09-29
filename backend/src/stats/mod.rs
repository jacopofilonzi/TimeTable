//! Usage log in SQLite: who fetched what, when.
//!
//! Tracked requests (see `state/tracking.rs` for which ones) become rows of `requests`: time, kind
//! (`feed` for `.ics`, `api` for JSON called from outside the wizard), endpoint, track id (`k`
//! query parameter, `NULL` when anonymous), university, schema params, weeks, custom name, client
//! family, raw User-Agent and HTTP status. Successful feeds with a track id also update
//! `feed_subscribers` (one row per id: first/last seen, count, last client and params), which
//! is kept forever; `requests` rows older than `FEED_STATS_RETENTION_DAYS` are purged daily.
//!
//! Enabled by `FEED_STATS_DB` (see `config`). Like the short link database it is durable data:
//! if it can't be opened or written at startup, the server refuses to start. Rows are queued and
//! written by a background thread ([`writer`]), so requests never wait on SQLite; a full queue
//! drops rows (counted in `timetable_tracking_dropped_total`).

mod client;
mod event;
mod store;
mod writer;

use std::{
    path::Path,
    sync::{Arc, Mutex, mpsc::SyncSender},
    time::{Duration, Instant},
};

use chrono::Utc;
use rusqlite::Connection;

pub use client::Client;
pub use event::{Kind, RequestEvent, parse_track_id};
pub use store::{RequestFilter, RequestRow, SubscriberRow, Summary};

use crate::{
    errors::{AppError, Internal},
    metrics::{self, DbLabels},
};

/// How often old requests are purged.
const PURGE_INTERVAL: Duration = Duration::from_secs(24 * 3600);
/// How long [`FeedStats::summary`] results are reused (Prometheus scrapes often).
const SUMMARY_TTL: Duration = Duration::from_secs(30);

#[derive(Debug, thiserror::Error)]
pub enum OpenError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error("the database is read-only")]
    ReadOnly,
}

#[derive(Clone)]
pub struct FeedStats {
    db: Arc<Mutex<Connection>>,
    queue: SyncSender<writer::Message>,
    summary: Arc<Mutex<Option<(Instant, Summary)>>>,
}

impl FeedStats {
    /// Opens (creating it and its directory if needed) the database at `path`, checks that it can
    /// be written, and starts the writer and, if `retention_days` is set, the daily purge.
    pub fn open(path: &Path, retention_days: Option<u32>) -> Result<Self, OpenError> {
        if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(dir)?;
        }
        let conn = Connection::open(path)?;
        conn.busy_timeout(Duration::from_secs(5))?;
        if conn.is_readonly(rusqlite::MAIN_DB)? {
            return Err(OpenError::ReadOnly);
        }
        store::init(&conn)?;
        let db = Arc::new(Mutex::new(conn));
        let stats = Self {
            queue: writer::spawn(db.clone())?,
            db,
            summary: Arc::new(Mutex::new(None)),
        };
        if let Some(days) = retention_days {
            stats.spawn_purge(days);
        }
        Ok(stats)
    }

    /// Queues a request for the log; never blocks.
    pub fn record(&self, event: RequestEvent) {
        writer::send(&self.queue, writer::Message::Row(Box::new(event)));
    }

    /// Waits until every request queued so far is written (used on shutdown).
    pub async fn flush(&self) {
        let (done, wait) = std::sync::mpsc::channel();
        writer::send(&self.queue, writer::Message::Flush(done));
        let _ =
            tokio::task::spawn_blocking(move || wait.recv_timeout(Duration::from_secs(5))).await;
    }

    /// Requests matching `filter`, newest first.
    pub async fn requests(&self, filter: RequestFilter) -> Result<Vec<RequestRow>, AppError> {
        self.run(move |conn| store::requests(conn, &filter)).await
    }

    /// Subscribers seen in the last `active_days` (all if `None`), most recent first.
    pub async fn subscribers(
        &self,
        active_days: Option<u32>,
        limit: u32,
    ) -> Result<Vec<SubscriberRow>, AppError> {
        let since = active_days.map(|d| Utc::now().timestamp() - i64::from(d) * 86_400);
        self.run(move |conn| store::subscribers(conn, since, limit))
            .await
    }

    /// Aggregates for the Prometheus gauges, recomputed at most every [`SUMMARY_TTL`].
    pub async fn summary(&self) -> Result<Summary, AppError> {
        {
            let cached = self.summary.lock().unwrap_or_else(|p| p.into_inner());
            if let Some((at, summary)) = cached.as_ref()
                && at.elapsed() < SUMMARY_TTL
            {
                return Ok(summary.clone());
            }
        }
        let summary = self
            .run(|conn| store::summary(conn, Utc::now().timestamp()))
            .await?;
        *self.summary.lock().unwrap_or_else(|p| p.into_inner()) =
            Some((Instant::now(), summary.clone()));
        Ok(summary)
    }

    fn spawn_purge(&self, days: u32) {
        let this = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(PURGE_INTERVAL);
            loop {
                interval.tick().await;
                let before = Utc::now().timestamp() - i64::from(days) * 86_400;
                match this.run(move |conn| store::purge(conn, before)).await {
                    Ok(0) => {}
                    Ok(n) => tracing::info!(rows = n, days, "purged old usage log rows"),
                    Err(err) => tracing::warn!(%err, "could not purge the usage log"),
                }
            }
        });
    }

    /// Runs `f` on a blocking thread with the connection.
    async fn run<T, F>(&self, f: F) -> Result<T, AppError>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection) -> rusqlite::Result<T> + Send + 'static,
    {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = db.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            f(&mut conn)
        })
        .await
        .map_err(|e| Internal::new(format!("usage log task failed: {e}")))?
        .map_err(|e| {
            metrics::get()
                .sqlite_errors
                .get_or_create(&DbLabels { db: "feed_stats" })
                .inc();
            Internal::new(format!("usage log database: {e}")).into()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn records_through_the_writer() {
        let dir = std::env::temp_dir().join(format!("tt-feedstats-{}", std::process::id()));
        let path = dir.join("nested").join("stats.db");
        let stats = FeedStats::open(&path, Some(365)).unwrap();
        stats.record(RequestEvent {
            at: Utc::now().timestamp(),
            kind: Kind::Feed,
            endpoint: "lessons.ics".into(),
            track_id: Some("aZ3x9QbT".into()),
            short_code: None,
            university: Some("unicam"),
            params: None,
            weeks: None,
            name: None,
            client: Client::Apple,
            user_agent: None,
            status: 200,
        });
        stats.flush().await;
        let rows = stats
            .requests(RequestFilter {
                limit: 10,
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].track_id.as_deref(), Some("aZ3x9QbT"));
        assert_eq!(stats.subscribers(Some(1), 10).await.unwrap().len(), 1);
        assert!(path.is_file());
        drop(stats);
        std::fs::remove_dir_all(dir).ok();
    }
}
