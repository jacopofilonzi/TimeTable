//! One tracked request, as written to the `requests` table.

use super::Client;

/// Longest accepted track id (`k`); shorter than [`MIN_TRACK_ID`] or non-alphanumeric ids are
/// ignored (the request counts as anonymous). The frontend generates 8 characters.
const MAX_TRACK_ID: usize = 32;
const MIN_TRACK_ID: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// `.ics` feeds: user usage (calendar apps).
    Feed,
    /// JSON endpoints called from outside the wizard.
    Api,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Feed => "feed",
            Self::Api => "api",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "feed" => Some(Self::Feed),
            "api" => Some(Self::Api),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RequestEvent {
    /// Unix seconds.
    pub at: i64,
    pub kind: Kind,
    /// `lessons.ics`, `short.ics`, `universities`, `university`, `options:<field>`, `lessons`.
    pub endpoint: String,
    /// `None` for anonymous requests.
    pub track_id: Option<String>,
    pub short_code: Option<String>,
    pub university: Option<&'static str>,
    /// Declared schema params present in the request, canonical (`course=…&year=…`).
    pub params: Option<String>,
    pub weeks: Option<u8>,
    /// Custom calendar name; `None` when the default one is used.
    pub name: Option<String>,
    pub client: Client,
    pub user_agent: Option<String>,
    /// HTTP status of the response.
    pub status: u16,
}

impl RequestEvent {
    /// A request that should count for `feed_subscribers`.
    pub fn is_subscriber_fetch(&self) -> bool {
        self.kind == Kind::Feed && self.track_id.is_some() && self.status < 400
    }
}

/// The `k` query parameter, if it is a well-formed track id.
pub fn parse_track_id(value: Option<&str>) -> Option<String> {
    value
        .filter(|v| (MIN_TRACK_ID..=MAX_TRACK_ID).contains(&v.len()))
        .filter(|v| v.bytes().all(|b| b.is_ascii_alphanumeric()))
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_track_ids() {
        assert_eq!(
            parse_track_id(Some("aZ3x9QbT")).as_deref(),
            Some("aZ3x9QbT")
        );
        assert_eq!(parse_track_id(None), None);
        assert_eq!(parse_track_id(Some("")), None);
        assert_eq!(parse_track_id(Some("short")), None);
        assert_eq!(parse_track_id(Some("aZ3x9Qb-")), None);
        assert_eq!(parse_track_id(Some("aZ3x9QbT'; DROP")), None);
        assert_eq!(parse_track_id(Some(&"a".repeat(33))), None);
    }
}
