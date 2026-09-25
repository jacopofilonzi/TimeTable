//! In-memory cache entries: type-erased values with their own TTL.

use std::{
    any::Any,
    sync::Arc,
    time::{Duration, Instant},
};

use moka::Expiry;

#[derive(Clone)]
pub struct Entry {
    value: Arc<dyn Any + Send + Sync>,
    ttl: Duration,
}

impl Entry {
    pub fn new<T: Send + Sync + 'static>(value: T, ttl: Duration) -> Self {
        Self {
            value: Arc::new(value),
            ttl,
        }
    }

    /// The stored value, if it has type `T`.
    pub fn downcast<T: Send + Sync + 'static>(self) -> Option<Arc<T>> {
        self.value.downcast::<T>().ok()
    }
}

/// Expires each entry after its own TTL.
pub struct PerEntryTtl;

impl Expiry<String, Entry> for PerEntryTtl {
    fn expire_after_create(
        &self,
        _key: &String,
        entry: &Entry,
        _created_at: Instant,
    ) -> Option<Duration> {
        Some(entry.ttl)
    }
}
