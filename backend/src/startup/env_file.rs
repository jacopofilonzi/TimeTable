use std::path::PathBuf;

/// Outcome of loading `.env`. It is read before logging is set up, so it is logged afterwards.
pub enum EnvFile {
    Loaded(PathBuf),
    Missing,
    /// A line couldn't be parsed: loading stopped there, it and the lines after it were ignored.
    Invalid {
        key: Option<String>,
        index: usize,
    },
    Unreadable(String),
}

/// Loads `.env` from the current directory or its parents (so the repo root `.env` works when
/// running from `backend/`). Real environment variables take precedence.
pub fn load_env_file() -> EnvFile {
    match dotenvy::dotenv() {
        Ok(path) => EnvFile::Loaded(path),
        Err(err) if err.not_found() => EnvFile::Missing,
        // dotenvy reports the offending value, not the key: find the key in the file ourselves.
        Err(dotenvy::Error::LineParse(value, index)) => EnvFile::Invalid {
            key: find_env_file()
                .and_then(|path| std::fs::read_to_string(path).ok())
                .and_then(|contents| key_for_value(&contents, &value)),
            index,
        },
        Err(err) => EnvFile::Unreadable(err.to_string()),
    }
}

impl EnvFile {
    /// Never logs values: they may be secrets (`AUTH_TOKEN`).
    pub fn log(&self) {
        match self {
            Self::Loaded(path) => tracing::info!("loaded environment from {}", path.display()),
            Self::Missing => {}
            Self::Invalid { key, index } => tracing::warn!(
                "invalid line in .env ({}, character {index}): it and the lines after it were \
                 not loaded. Quote values containing backslashes ('C:\\dir') or use '/'",
                key.as_deref()
                    .map_or("unknown variable".to_string(), |k| format!("variable {k}"))
            ),
            Self::Unreadable(err) => tracing::warn!("could not load .env: {err}"),
        }
    }
}

/// The `.env` dotenvy picks: the first one in the current directory or its parents.
fn find_env_file() -> Option<PathBuf> {
    let dir = std::env::current_dir().ok()?;
    dir.ancestors()
        .map(|d| d.join(".env"))
        .find(|p| p.is_file())
}

/// Key of the `KEY=value` line whose value is `value`.
fn key_for_value(contents: &str, value: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let (key, v) = line.split_once('=')?;
        let key = key.trim().trim_start_matches("export ").trim();
        (v.trim() == value.trim() && !key.starts_with('#')).then(|| key.to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::key_for_value;

    #[test]
    fn finds_the_key_of_a_value() {
        let env = "# comment=x\nPORT=8080\nSHORTLINK_DB=C:\\dir\\a.db\nexport TOKEN=abc\n";
        assert_eq!(
            key_for_value(env, "C:\\dir\\a.db").as_deref(),
            Some("SHORTLINK_DB")
        );
        assert_eq!(key_for_value(env, "abc").as_deref(), Some("TOKEN"));
        assert_eq!(key_for_value(env, "x"), None);
        assert_eq!(key_for_value(env, "missing"), None);
    }
}
