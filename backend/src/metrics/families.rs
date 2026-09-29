//! The metric families and their registration.

use std::sync::atomic::AtomicU64;

use prometheus_client::{
    metrics::{
        counter::Counter,
        family::Family,
        gauge::Gauge,
        histogram::{Histogram, exponential_buckets},
        info::Info,
    },
    registry::Registry,
};

use super::{labels::*, process::ProcessCollector};

type HistogramFamily<L> = Family<L, Histogram, fn() -> Histogram>;

pub struct Metrics {
    pub(super) registry: Registry,

    pub http_requests: Family<HttpLabels, Counter>,
    pub http_duration: HistogramFamily<RouteLabels>,

    pub cache_requests: Family<CacheLabels, Counter>,
    pub cache_entries: Gauge,
    pub redis_status: Family<RedisStatusLabels, Gauge>,
    pub redis_errors: Family<RedisErrorLabels, Counter>,

    pub upstream_requests: Family<UpstreamResultLabels, Counter>,
    pub upstream_duration: HistogramFamily<UpstreamLabels>,
    pub upstream_last_lessons: Family<UniversityLabels, Gauge>,

    pub short_links_created: Counter,
    pub short_link_opens: Family<ShortLinkOpenLabels, Counter>,
    pub short_links: Gauge,

    pub sqlite_errors: Family<DbLabels, Counter>,
    pub sqlite_file_bytes: Family<DbLabels, Gauge>,
    pub tracking_dropped: Counter,
    pub auth_failures: Family<AuthLabels, Counter>,

    pub tracked_requests: Family<TrackedRequestLabels, Counter>,
    pub feed_subscribers: Family<UniversityLabels, Gauge>,
    pub feed_subscribers_active: Family<WindowLabels, Gauge>,
    pub feed_subscribers_by_params: Family<ParamsLabels, Gauge>,
    pub feed_anonymous_estimate: Family<WindowLabels, Gauge>,

    pub process_start_time: Gauge<f64, AtomicU64>,
}

impl Metrics {
    pub fn new() -> Self {
        let mut registry = Registry::default();
        let m = Self {
            registry: Registry::default(),
            http_requests: Family::default(),
            http_duration: Family::new_with_constructor(|| {
                Histogram::new(exponential_buckets(0.005, 2.0, 12))
            }),
            cache_requests: Family::default(),
            cache_entries: Gauge::default(),
            redis_status: Family::default(),
            redis_errors: Family::default(),
            upstream_requests: Family::default(),
            upstream_duration: Family::new_with_constructor(|| {
                Histogram::new(exponential_buckets(0.05, 2.0, 10))
            }),
            upstream_last_lessons: Family::default(),
            short_links_created: Counter::default(),
            short_link_opens: Family::default(),
            short_links: Gauge::default(),
            sqlite_errors: Family::default(),
            sqlite_file_bytes: Family::default(),
            tracking_dropped: Counter::default(),
            auth_failures: Family::default(),
            tracked_requests: Family::default(),
            feed_subscribers: Family::default(),
            feed_subscribers_active: Family::default(),
            feed_subscribers_by_params: Family::default(),
            feed_anonymous_estimate: Family::default(),
            process_start_time: Gauge::default(),
        };

        let r = &mut registry;
        // Counters get `_total` appended by the encoder.
        r.register(
            "timetable_http_requests",
            "HTTP requests by route template, method and status",
            m.http_requests.clone(),
        );
        r.register(
            "timetable_http_request_duration_seconds",
            "HTTP request duration by route template and method",
            m.http_duration.clone(),
        );
        r.register(
            "timetable_cache_requests",
            "Cache lookups by kind and result (memory_hit, redis_hit, miss, error)",
            m.cache_requests.clone(),
        );
        r.register(
            "timetable_cache_entries",
            "Entries in the in-memory cache",
            m.cache_entries.clone(),
        );
        r.register(
            "timetable_redis_status",
            "Redis status: 1 for the current one (disabled, connected, disconnected)",
            m.redis_status.clone(),
        );
        r.register(
            "timetable_redis_errors",
            "Redis errors and timeouts (each degraded to a cache miss)",
            m.redis_errors.clone(),
        );
        r.register(
            "timetable_upstream_requests",
            "Fetches from the university sites by operation and result",
            m.upstream_requests.clone(),
        );
        r.register(
            "timetable_upstream_duration_seconds",
            "Duration of the fetches from the university sites",
            m.upstream_duration.clone(),
        );
        r.register(
            "timetable_upstream_last_lessons",
            "Lessons returned by the last successful lessons fetch",
            m.upstream_last_lessons.clone(),
        );
        r.register(
            "timetable_short_links_created",
            "Successful short link requests (POST /api/short, new or existing code)",
            m.short_links_created.clone(),
        );
        r.register(
            "timetable_short_link_opens",
            "Short links opened, by target, origin (QR or link) and language",
            m.short_link_opens.clone(),
        );
        r.register(
            "timetable_short_links",
            "Short links stored in the database",
            m.short_links.clone(),
        );
        r.register(
            "timetable_sqlite_errors",
            "Failed SQLite operations by database",
            m.sqlite_errors.clone(),
        );
        r.register(
            "timetable_sqlite_file_bytes",
            "Size of the SQLite files (database + WAL) by database",
            m.sqlite_file_bytes.clone(),
        );
        r.register(
            "timetable_tracking_dropped",
            "Usage log rows dropped because the write queue was full",
            m.tracking_dropped.clone(),
        );
        r.register(
            "timetable_auth_failures",
            "Rejected requests to protected endpoints, by token scope",
            m.auth_failures.clone(),
        );
        r.register(
            "timetable_tracked_requests",
            "Tracked requests: feed (.ics) or api (JSON outside the wizard), with/without track id, by client",
            m.tracked_requests.clone(),
        );
        r.register(
            "timetable_feed_subscribers",
            "Track ids ever seen on feeds (all-time subscribers)",
            m.feed_subscribers.clone(),
        );
        r.register(
            "timetable_feed_subscribers_active",
            "Track ids that fetched their feed within the window",
            m.feed_subscribers_active.clone(),
        );
        r.register(
            "timetable_feed_subscribers_by_params",
            "Track ids active in the last 7 days, by feed settings",
            m.feed_subscribers_by_params.clone(),
        );
        r.register(
            "timetable_feed_anonymous_estimate",
            "Lower bound of distinct anonymous feeds within the window (distinct settings + name + client)",
            m.feed_anonymous_estimate.clone(),
        );
        r.register(
            "process_start_time_seconds",
            "Start time of the process since the Unix epoch, in seconds",
            m.process_start_time.clone(),
        );
        r.register(
            "timetable_build",
            "Build information",
            Info::new(vec![("version", env!("CARGO_PKG_VERSION"))]),
        );
        r.register_collector(Box::new(ProcessCollector));

        m.process_start_time
            .set(chrono::Utc::now().timestamp_millis() as f64 / 1000.0);
        Self { registry, ..m }
    }
}
