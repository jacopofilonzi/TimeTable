//! Which requests are tracked, and how they become usage log rows and metrics.
//!
//! - `.ics` feeds (`lessons.ics`, `/s/{code}.ics`) are always tracked, as `feed`.
//! - Public JSON endpoints are tracked as `api`, except when they come from our own wizard, which
//!   marks its requests with [`OWN_FRONTEND_HEADER`]: those are not tracked at all.
//! - Anything else (admin, stats, metrics, short link creation, static files) is not tracked.
//!
//! The track id is the `k` query parameter (see `stats::parse_track_id`); without a valid one the
//! request is anonymous.

use std::collections::HashMap;

use axum::http::{HeaderMap, StatusCode, header};
use chrono::Utc;

use super::{
    AppState,
    validation::{MAX_VALUE_LEN, all_fields, canonical},
};
use crate::{
    metrics::{self, TrackedRequestLabels},
    models::UniversityInfo,
    stats::{Client, Kind, RequestEvent, parse_track_id},
    universities::Params,
};

/// Header the frontend adds to its API calls (value [`OWN_FRONTEND_VALUE`]).
pub const OWN_FRONTEND_HEADER: &str = "x-tt-client";
pub const OWN_FRONTEND_VALUE: &str = "web";
/// Longest stored custom calendar name and User-Agent, in characters.
const MAX_NAME_CHARS: usize = 100;
const MAX_USER_AGENT_CHARS: usize = 300;

/// A tracked endpoint.
#[derive(Debug, Clone, Copy)]
pub enum Endpoint<'a> {
    LessonsIcs,
    /// `/s/{code}.ics`, with the normalized code.
    ShortIcs(&'a str),
    Universities,
    University,
    /// `/options/{field}`, with the field as requested.
    Options(&'a str),
    Lessons,
}

impl Endpoint<'_> {
    fn kind(self) -> Kind {
        match self {
            Self::LessonsIcs | Self::ShortIcs(_) => Kind::Feed,
            _ => Kind::Api,
        }
    }

    /// Stored name; option fields only when declared, so junk paths don't reach the log.
    fn name(self, info: Option<&UniversityInfo>) -> String {
        match self {
            Self::LessonsIcs => "lessons.ics".into(),
            Self::ShortIcs(_) => "short.ics".into(),
            Self::Universities => "universities".into(),
            Self::University => "university".into(),
            Self::Options(field) => match info.and_then(|i| all_fields(i).find(|f| f.key == field))
            {
                Some(field) => format!("options:{}", field.key),
                None => "options".into(),
            },
            Self::Lessons => "lessons".into(),
        }
    }
}

/// A finished request to a tracked endpoint.
pub struct TrackedRequest<'a> {
    pub endpoint: Endpoint<'a>,
    /// University id as requested (resolved through the registry; unknown ids are stored as NULL).
    pub university: Option<&'a str>,
    /// Query parameters: schema fields, `weeks`, `name`, `k`.
    pub query: &'a HashMap<String, String>,
    pub headers: &'a HeaderMap,
    pub status: StatusCode,
}

impl AppState {
    /// Records a finished request in the metrics and the usage log (if enabled). Never fails.
    pub fn track(&self, req: TrackedRequest<'_>) {
        let kind = req.endpoint.kind();
        if kind == Kind::Api && from_own_frontend(req.headers) {
            return;
        }
        let user_agent = req
            .headers
            .get(header::USER_AGENT)
            .and_then(|v| v.to_str().ok());
        let client = Client::from_user_agent(user_agent);
        let track_id = parse_track_id(req.query.get("k").map(String::as_str));

        metrics::get()
            .tracked_requests
            .get_or_create(&TrackedRequestLabels {
                kind: kind.as_str(),
                tracked: if track_id.is_some() { "true" } else { "false" },
                client: client.as_str(),
            })
            .inc();

        let Some(stats) = &self.feed_stats else {
            return;
        };
        let info = req
            .university
            .and_then(|id| self.registry.get(id).ok())
            .map(|u| u.info());
        let name = match kind {
            Kind::Feed => req
                .query
                .get("name")
                .map(|n| n.trim())
                .filter(|n| !n.is_empty())
                .map(|n| n.chars().take(MAX_NAME_CHARS).collect()),
            Kind::Api => None,
        };
        stats.record(RequestEvent {
            at: Utc::now().timestamp(),
            kind,
            endpoint: req.endpoint.name(info),
            track_id,
            short_code: match req.endpoint {
                Endpoint::ShortIcs(code) => Some(code.to_string()),
                _ => None,
            },
            university: info.map(|i| i.id),
            params: info.and_then(|i| schema_params(i, req.query)),
            weeks: req.query.get("weeks").and_then(|w| w.parse().ok()),
            name,
            client,
            user_agent: user_agent.map(|ua| ua.chars().take(MAX_USER_AGENT_CHARS).collect()),
            status: req.status.as_u16(),
        });
    }
}

fn from_own_frontend(headers: &HeaderMap) -> bool {
    headers
        .get(OWN_FRONTEND_HEADER)
        .is_some_and(|v| v.as_bytes() == OWN_FRONTEND_VALUE.as_bytes())
}

/// The declared schema fields present in `query`, canonical; values are only length-capped (the
/// request may have failed validation, and the log should show what was asked).
fn schema_params(info: &UniversityInfo, query: &HashMap<String, String>) -> Option<String> {
    let params: Params = all_fields(info)
        .filter_map(|f| {
            let value = query.get(f.key)?;
            Some((
                f.key.to_string(),
                value.chars().take(MAX_VALUE_LEN).collect(),
            ))
        })
        .collect();
    (!params.is_empty()).then(|| canonical(&params))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universities::Registry;

    #[test]
    fn own_frontend_header() {
        let mut headers = HeaderMap::new();
        assert!(!from_own_frontend(&headers));
        headers.insert(OWN_FRONTEND_HEADER, "web".parse().unwrap());
        assert!(from_own_frontend(&headers));
        headers.insert(OWN_FRONTEND_HEADER, "curl".parse().unwrap());
        assert!(!from_own_frontend(&headers));
    }

    #[test]
    fn keeps_only_declared_params() {
        let registry = Registry::new();
        let info = registry.get("unicam").unwrap().info();
        let keys: Vec<&str> = all_fields(info).map(|f| f.key).collect();
        let mut query: HashMap<String, String> = keys
            .iter()
            .map(|k| (k.to_string(), "1".to_string()))
            .collect();
        query.insert("k".into(), "aZ3x9QbT".into());
        query.insert("evil".into(), "x".into());
        let params = schema_params(info, &query).unwrap();
        assert!(!params.contains("evil") && !params.contains("k="));
        assert_eq!(params.matches('=').count(), keys.len());
        assert_eq!(schema_params(info, &HashMap::new()), None);
    }

    #[test]
    fn option_endpoints_only_name_declared_fields() {
        let registry = Registry::new();
        let info = registry.get("unicam").unwrap().info();
        let field = all_fields(info).next().unwrap().key;
        assert_eq!(
            Endpoint::Options(field).name(Some(info)),
            format!("options:{field}")
        );
        assert_eq!(Endpoint::Options("junk").name(Some(info)), "options");
    }
}
