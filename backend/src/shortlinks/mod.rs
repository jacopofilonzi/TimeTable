//! Short links (`/s/{code}`) to a wizard page with its settings, stored in SQLite.
//!
//! A link maps a code to the canonical query of the page (`uni=…&<fields>&weeks=N`, validated and
//! URL-encoded). The code is derived from a hash of that query (see [`code`]), so the same
//! settings always get the same link. Codes never change once created, which is why lookups can
//! go through the regular cache.
//!
//! Enabled by `SHORTLINK_DB` (see `config`). Unlike Redis, the database is required when enabled:
//! if it can't be opened or written, the server refuses to start.

mod code;
mod store;

use std::{
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};

use chrono::Utc;
use rusqlite::Connection;

pub use code::normalize;

use crate::errors::{AppError, Internal};

/// `last_used_at` is updated at most once per link in this interval.
const TOUCH_INTERVAL: Duration = Duration::from_secs(24 * 3600);
const TOUCH_CAPACITY: u64 = 100_000;

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
pub struct ShortLinks {
    db: Arc<Mutex<Connection>>,
    /// Links whose `last_used_at` was updated recently.
    touched: moka::future::Cache<String, ()>,
}

impl ShortLinks {
    /// Opens (creating it and its directory if needed) the database at `path`, and checks that it
    /// can be written.
    pub fn open(path: &Path) -> Result<Self, OpenError> {
        if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(dir)?;
        }
        let conn = Connection::open(path)?;
        conn.busy_timeout(Duration::from_secs(5))?;
        if conn.is_readonly(rusqlite::MAIN_DB)? {
            return Err(OpenError::ReadOnly);
        }
        store::init(&conn)?;
        Ok(Self {
            db: Arc::new(Mutex::new(conn)),
            touched: moka::future::Cache::builder()
                .max_capacity(TOUCH_CAPACITY)
                .time_to_live(TOUCH_INTERVAL)
                .build(),
        })
    }

    /// The code for a validated canonical query, creating it if needed.
    pub async fn create(&self, canonical: String) -> Result<String, AppError> {
        self.run(move |conn| store::insert_or_get(conn, &canonical, Utc::now().timestamp()))
            .await?
            .ok_or_else(|| Internal::new("no free short link code").into())
    }

    /// The canonical query of a normalized code.
    pub async fn lookup(&self, code: &str) -> Result<Option<String>, AppError> {
        let code = code.to_string();
        self.run(move |conn| store::lookup(conn, &code)).await
    }

    /// Records a visit, at most once per [`TOUCH_INTERVAL`]; runs in the background and only
    /// logs failures.
    pub async fn touch(&self, code: &str) {
        if self.touched.contains_key(code) {
            return;
        }
        self.touched.insert(code.to_string(), ()).await;
        let this = self.clone();
        let code = code.to_string();
        tokio::spawn(async move {
            let result = this
                .run(move |conn| store::touch(conn, &code, Utc::now().timestamp()))
                .await;
            if let Err(err) = result {
                tracing::warn!(%err, "could not update short link last_used_at");
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
        .map_err(|e| Internal::new(format!("short link task failed: {e}")))?
        .map_err(|e| Internal::new(format!("short link database: {e}")).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn open_creates_directory_and_file() {
        let dir = std::env::temp_dir().join(format!("tt-shortlinks-{}", std::process::id()));
        let path = dir.join("nested").join("links.db");
        let links = ShortLinks::open(&path).unwrap();
        let code = links.create("uni=unicam&weeks=4".into()).await.unwrap();
        assert_eq!(
            links.lookup(&code).await.unwrap().as_deref(),
            Some("uni=unicam&weeks=4")
        );
        assert!(path.is_file());
        drop(links);
        std::fs::remove_dir_all(dir).ok();
    }
}
