//! Two-level cache: an always-on in-memory layer (moka) backed by an optional Redis layer.
//!
//! - The in-memory layer coalesces concurrent requests for the same key, so N simultaneous
//!   misses produce a single upstream fetch.
//! - Redis (when `REDIS_URL` is set and reachable) survives restarts and can be shared between
//!   instances. Every Redis failure degrades to "miss" and never fails the request.

mod entry;
mod redis_layer;

use std::{future::Future, sync::Arc, time::Duration};

use moka::future::Cache as MemCache;
use serde::{Serialize, de::DeserializeOwned};

pub use self::redis_layer::RedisClear;
use self::{
    entry::{Entry, PerEntryTtl},
    redis_layer::RedisLayer,
};
use crate::errors::{AppError, Internal};

/// Every key this app writes to Redis starts with this; [`Cache::clear`] deletes all of them.
const KEY_NAMESPACE: &str = "timetable:";
/// Bump the version when cached payload shapes change, so stale entries are ignored instead of
/// failing to decode. Must start with [`KEY_NAMESPACE`].
const KEY_PREFIX: &str = "timetable:v1:";
const MEMORY_CAPACITY: u64 = 10_000;

#[derive(Clone)]
pub struct Cache {
    memory: MemCache<String, Entry>,
    redis: RedisLayer,
}

impl Cache {
    /// Creates the cache. If a Redis URL is given, connects in the background (retrying until it
    /// succeeds), so the server starts even if Redis is down.
    pub fn new(redis_url: Option<&str>) -> Self {
        Self {
            memory: MemCache::builder()
                .max_capacity(MEMORY_CAPACITY)
                .expire_after(PerEntryTtl)
                .build(),
            redis: RedisLayer::new(redis_url),
        }
    }

    pub fn redis_connected(&self) -> bool {
        self.redis.is_connected()
    }

    /// Empties the in-memory cache and deletes this app's keys from Redis (every version).
    pub async fn clear(&self) -> ClearReport {
        self.memory.run_pending_tasks().await;
        let memory = self.memory.entry_count();
        self.memory.invalidate_all();
        self.memory.run_pending_tasks().await;
        ClearReport {
            memory,
            redis: self.redis.clear(KEY_NAMESPACE).await,
        }
    }

    /// Returns the cached value for `key`, or runs `fetch` and caches its result for `ttl`.
    /// Errors are not cached.
    pub async fn get_or_fetch<T, F, Fut>(
        &self,
        key: &str,
        ttl: Duration,
        fetch: F,
    ) -> Result<Arc<T>, AppError>
    where
        T: Serialize + DeserializeOwned + Send + Sync + 'static,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, AppError>>,
    {
        let key = format!("{KEY_PREFIX}{key}");
        let entry = self
            .memory
            .try_get_with(key.clone(), async {
                if let Some((value, remaining)) = self.redis.get::<T>(&key).await {
                    tracing::debug!(%key, "cache hit (redis)");
                    return Ok(Entry::new(value, remaining.min(ttl)));
                }
                tracing::debug!(%key, "cache miss");
                let value = fetch().await?;
                self.redis.set(&key, &value, ttl).await;
                Ok(Entry::new(value, ttl))
            })
            .await
            .map_err(|e: Arc<AppError>| (*e).clone())?;

        entry
            .downcast::<T>()
            .ok_or_else(|| Internal::new(format!("cache type mismatch for key {key}")).into())
    }
}

/// What [`Cache::clear`] removed.
#[derive(Debug, Serialize)]
pub struct ClearReport {
    /// In-memory entries removed.
    pub memory: u64,
    pub redis: RedisClear,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::Upstream;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn coalesces_and_caches() {
        let cache = Cache::new(None);
        let calls = Arc::new(AtomicUsize::new(0));

        let fetch = || {
            let calls = calls.clone();
            async move {
                calls.fetch_add(1, Ordering::SeqCst);
                tokio::time::sleep(Duration::from_millis(50)).await;
                Ok::<_, AppError>(vec![1, 2, 3])
            }
        };

        let ttl = Duration::from_secs(60);
        let (a, b) = tokio::join!(
            cache.get_or_fetch("k", ttl, fetch),
            cache.get_or_fetch("k", ttl, fetch)
        );
        assert_eq!(*a.unwrap(), vec![1, 2, 3]);
        assert_eq!(*b.unwrap(), vec![1, 2, 3]);
        cache.get_or_fetch("k", ttl, fetch).await.unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn key_prefix_is_namespaced() {
        assert!(KEY_PREFIX.starts_with(KEY_NAMESPACE));
    }

    #[tokio::test]
    async fn clear_empties_memory() {
        let cache = Cache::new(None);
        let ttl = Duration::from_secs(60);
        cache
            .get_or_fetch("a", ttl, || async { Ok::<_, AppError>(1u8) })
            .await
            .unwrap();
        let report = cache.clear().await;
        assert_eq!(report.memory, 1);
        assert!(matches!(report.redis, RedisClear::Disabled));

        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let c = calls.clone();
        cache
            .get_or_fetch("a", ttl, || async move {
                c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok::<_, AppError>(1u8)
            })
            .await
            .unwrap();
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn errors_are_not_cached() {
        let cache = Cache::new(None);
        let ttl = Duration::from_secs(60);
        let err = cache
            .get_or_fetch::<Vec<u8>, _, _>("e", ttl, || async { Err(Upstream::msg("down").into()) })
            .await;
        assert!(err.is_err());
        let ok = cache
            .get_or_fetch("e", ttl, || async { Ok::<_, AppError>(vec![1u8]) })
            .await;
        assert_eq!(*ok.unwrap(), vec![1u8]);
    }
}
