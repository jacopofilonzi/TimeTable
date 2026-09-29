use std::path::PathBuf;

use super::env;

/// Used when `SHORTLINK_DB` is not set (relative to the working directory; the docker image sets
/// `/data/shortlinks.db`).
const DEFAULT: &str = "data/shortlinks.db";

/// `SHORTLINK_DB`: a file path, `false` to disable short links, unset for [`DEFAULT`].
pub fn resolve() -> Option<PathBuf> {
    from_value(env::var("SHORTLINK_DB").as_deref())
}

fn from_value(value: Option<&str>) -> Option<PathBuf> {
    match value {
        None => Some(PathBuf::from(DEFAULT)),
        Some(v) if v.eq_ignore_ascii_case("false") => None,
        Some(v) => Some(PathBuf::from(v)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_values() {
        assert_eq!(from_value(None), Some(PathBuf::from(DEFAULT)));
        assert_eq!(from_value(Some("False")), None);
        assert_eq!(
            from_value(Some("/data/x.db")),
            Some(PathBuf::from("/data/x.db"))
        );
    }
}
