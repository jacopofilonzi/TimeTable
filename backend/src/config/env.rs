//! Environment variable helpers.

/// Reads an env var, treating empty values as unset.
pub fn var(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

/// Reads and parses an env var; unset or unparsable values are `None`.
pub fn parse<T: std::str::FromStr>(key: &str) -> Option<T> {
    var(key).and_then(|v| v.parse().ok())
}
