use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lesson {
    /// Stable identifier from the source, used as the ICS UID.
    pub id: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub subject: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub teacher: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}
