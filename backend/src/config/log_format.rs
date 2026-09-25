/// Log output format (`LOG_FORMAT`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    Pretty,
    Compact,
    Json,
}

impl LogFormat {
    /// `json` | `compact` | anything else (default) = `pretty`.
    pub fn from_env_value(value: Option<&str>) -> Self {
        match value {
            Some("json") => Self::Json,
            Some("compact") => Self::Compact,
            _ => Self::Pretty,
        }
    }
}
