//! Label sets of the metric families.

use prometheus_client::encoding::EncodeLabelSet;

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct HttpLabels {
    /// Route template (`/api/universities/{id}/lessons`), `static` or `unmatched`.
    pub route: String,
    pub method: String,
    pub status: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct RouteLabels {
    pub route: String,
    pub method: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct CacheLabels {
    /// What is cached: `lessons`, `options`, `short`.
    pub kind: String,
    /// `memory_hit`, `redis_hit`, `miss` (fetched) or `error` (fetch failed).
    pub result: &'static str,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct RedisStatusLabels {
    /// `disabled`, `connected` or `disconnected`; the current one is 1, the others 0.
    pub status: &'static str,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct RedisErrorLabels {
    /// `get` or `set`.
    pub op: &'static str,
    /// `error` or `timeout`.
    pub reason: &'static str,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct UpstreamLabels {
    pub university: &'static str,
    /// `lessons` or `options`.
    pub operation: &'static str,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct UpstreamResultLabels {
    pub university: &'static str,
    pub operation: &'static str,
    /// `ok` or `error`.
    pub result: &'static str,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct UniversityLabels {
    pub university: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ShortLinkOpenLabels {
    /// `page` (redirect to the wizard) or `ics` (feed).
    pub target: &'static str,
    /// `qr` (uppercase `/S/`, as in the QR codes) or `link`.
    pub via: &'static str,
    /// Redirect language: `it`, `en`; `none` for feeds.
    pub lang: &'static str,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct DbLabels {
    /// `shortlinks` or `feed_stats`.
    pub db: &'static str,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct AuthLabels {
    /// `admin` (`AUTH_TOKEN`) or `stats` (`STATS_TOKEN`).
    pub scope: &'static str,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct TrackedRequestLabels {
    /// `feed` (`.ics`, user usage) or `api` (JSON from outside the wizard).
    pub kind: &'static str,
    /// Whether the request carried a valid `k` track id.
    pub tracked: &'static str,
    /// Client family from the User-Agent (`google`, `apple`, `curl`, …).
    pub client: &'static str,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct WindowLabels {
    pub university: String,
    /// `1d`, `7d` or `30d`.
    pub window: &'static str,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ParamsLabels {
    pub university: String,
    /// Canonical schema params (`course=…&year=…`); bounded by the university's options.
    pub params: String,
}
