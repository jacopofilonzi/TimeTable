//! Optional Redis layer. Connects in the background; until connected (or if `REDIS_URL` is not
//! set) every operation is a no-op/miss. Errors and timeouts are logged, never propagated (except
//! by `clear`, which reports them to the admin).

use std::{
    sync::{Arc, OnceLock},
    time::Duration,
};

use redis::{FromRedisValue, aio::ConnectionManager};
use serde::{Serialize, de::DeserializeOwned};

const TIMEOUT: Duration = Duration::from_millis(500);
/// Per-command timeout for admin operations (they may touch many keys).
const ADMIN_TIMEOUT: Duration = Duration::from_secs(5);
/// Keys per `SCAN` round when clearing.
const SCAN_COUNT: u32 = 500;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const RETRY: Duration = Duration::from_secs(15);

#[derive(Clone)]
pub struct RedisLayer {
    /// Whether `REDIS_URL` is set.
    enabled: bool,
    connection: Arc<OnceLock<ConnectionManager>>,
}

/// Outcome of [`RedisLayer::clear`], reported by the admin endpoint.
#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum RedisClear {
    /// `REDIS_URL` is not set.
    Disabled,
    /// Configured but not connected (yet).
    NotConnected,
    Cleared {
        keys: u64,
    },
    Failed {
        error: String,
    },
}

impl RedisLayer {
    pub fn new(url: Option<&str>) -> Self {
        let connection = Arc::new(OnceLock::new());
        match url {
            Some(url) => spawn_connector(url.to_string(), connection.clone()),
            None => tracing::info!("REDIS_URL not set, using in-memory cache only"),
        }
        Self {
            enabled: url.is_some(),
            connection,
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connection.get().is_some()
    }

    /// Value and remaining TTL, or `None` on miss, error, timeout or undecodable data.
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Option<(T, Duration)> {
        let mut conn = self.connection.get()?.clone();
        let mut pipe = redis::pipe();
        pipe.get(key).pttl(key);
        let query = pipe.query_async::<(Option<String>, i64)>(&mut conn);
        let (raw, pttl) = match tokio::time::timeout(TIMEOUT, query).await {
            Ok(Ok((Some(raw), pttl))) => (raw, pttl),
            Ok(Ok((None, _))) => return None,
            Ok(Err(err)) => {
                tracing::warn!(%err, "redis GET failed");
                return None;
            }
            Err(_) => {
                tracing::warn!("redis GET timed out");
                return None;
            }
        };
        match serde_json::from_str(&raw) {
            // pttl is -1 for keys without expiry; treat as "use the default ttl".
            Ok(value) => Some((
                value,
                Duration::from_millis(u64::try_from(pttl).unwrap_or(u64::MAX)),
            )),
            Err(err) => {
                tracing::warn!(%key, %err, "ignoring undecodable redis entry");
                None
            }
        }
    }

    pub async fn set<T: Serialize>(&self, key: &str, value: &T, ttl: Duration) {
        let Some(conn) = self.connection.get() else {
            return;
        };
        let raw = match serde_json::to_string(value) {
            Ok(raw) => raw,
            Err(err) => {
                tracing::warn!(%err, "failed to serialize cache entry");
                return;
            }
        };
        let mut conn = conn.clone();
        let mut cmd = redis::cmd("SET");
        cmd.arg(key).arg(raw).arg("EX").arg(ttl.as_secs().max(1));
        let query = cmd.query_async::<()>(&mut conn);
        match tokio::time::timeout(TIMEOUT, query).await {
            Ok(Ok(())) => {}
            Ok(Err(err)) => tracing::warn!(%err, "redis SET failed"),
            Err(_) => tracing::warn!("redis SET timed out"),
        }
    }

    /// Deletes every key starting with `prefix` (`SCAN` + `UNLINK`, never `FLUSHDB`, so other data
    /// in the same database is untouched).
    pub async fn clear(&self, prefix: &str) -> RedisClear {
        if !self.enabled {
            return RedisClear::Disabled;
        }
        let Some(conn) = self.connection.get() else {
            return RedisClear::NotConnected;
        };
        let mut conn = conn.clone();
        let pattern = format!("{prefix}*");
        let mut cursor: u64 = 0;
        let mut deleted: u64 = 0;
        loop {
            let mut scan = redis::cmd("SCAN");
            scan.arg(cursor)
                .arg("MATCH")
                .arg(&pattern)
                .arg("COUNT")
                .arg(SCAN_COUNT);
            let (next, keys): (u64, Vec<String>) = match admin_query(&scan, &mut conn).await {
                Ok(page) => page,
                Err(error) => return RedisClear::Failed { error },
            };
            if !keys.is_empty() {
                let mut unlink = redis::cmd("UNLINK");
                unlink.arg(&keys);
                match admin_query::<u64>(&unlink, &mut conn).await {
                    Ok(n) => deleted += n,
                    Err(error) => return RedisClear::Failed { error },
                }
            }
            if next == 0 {
                return RedisClear::Cleared { keys: deleted };
            }
            cursor = next;
        }
    }
}

/// Connects in the background, retrying every [`RETRY`] until it succeeds. Once connected, the
/// `ConnectionManager` handles reconnections on its own.
fn spawn_connector(url: String, slot: Arc<OnceLock<ConnectionManager>>) {
    tokio::spawn(async move {
        let client = match redis::Client::open(url) {
            Ok(client) => client,
            Err(err) => {
                tracing::error!(%err, "invalid REDIS_URL, using in-memory cache only");
                return;
            }
        };
        loop {
            match tokio::time::timeout(CONNECT_TIMEOUT, ConnectionManager::new(client.clone()))
                .await
            {
                Ok(Ok(manager)) => {
                    let _ = slot.set(manager);
                    tracing::info!("redis connected");
                    return;
                }
                Ok(Err(err)) => {
                    tracing::warn!(%err, "redis unavailable, retrying in {}s", RETRY.as_secs())
                }
                Err(_) => {
                    tracing::warn!(
                        "redis connection timed out, retrying in {}s",
                        RETRY.as_secs()
                    )
                }
            }
            tokio::time::sleep(RETRY).await;
        }
    });
}

/// Runs an admin command with [`ADMIN_TIMEOUT`], turning failures into a message.
async fn admin_query<T: FromRedisValue>(
    cmd: &redis::Cmd,
    conn: &mut ConnectionManager,
) -> Result<T, String> {
    match tokio::time::timeout(ADMIN_TIMEOUT, cmd.query_async::<T>(conn)).await {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(err)) => Err(err.to_string()),
        Err(_) => Err("timed out".into()),
    }
}
