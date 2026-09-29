use std::path::PathBuf;

use super::env;

/// `SHORTLINK_DB` when unset (relative to the working directory; the docker image sets
/// `/data/shortlinks.db`).
pub const SHORTLINK_DEFAULT: &str = "data/shortlinks.db";
/// `FEED_STATS_DB` when unset (the docker image sets `/data/feed_stats.db`).
pub const FEED_STATS_DEFAULT: &str = "data/feed_stats.db";

/// A SQLite file env var (`SHORTLINK_DB`, `FEED_STATS_DB`): a file path, `false` to disable the
/// feature, unset for `default`.
pub fn resolve(key: &str, default: &str) -> Option<PathBuf> {
    from_value(env::var(key).as_deref(), default)
}

fn from_value(value: Option<&str>, default: &str) -> Option<PathBuf> {
    match value {
        None => Some(PathBuf::from(default)),
        Some(v) if v.eq_ignore_ascii_case("false") => None,
        Some(v) => Some(PathBuf::from(v)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_values() {
        let default = SHORTLINK_DEFAULT;
        assert_eq!(from_value(None, default), Some(PathBuf::from(default)));
        assert_eq!(from_value(Some("False"), default), None);
        assert_eq!(
            from_value(Some("/data/x.db"), default),
            Some(PathBuf::from("/data/x.db"))
        );
    }
}
