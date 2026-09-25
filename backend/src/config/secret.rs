use std::fmt;

/// A secret string (e.g. `AUTH_TOKEN`). Its `Debug` output is redacted so it can't end up in logs.
#[derive(Clone)]
pub struct Secret(String);

impl Secret {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(***)")
    }
}

#[cfg(test)]
mod tests {
    use super::Secret;

    #[test]
    fn debug_is_redacted() {
        let secret = Secret::new("hunter2".into());
        assert_eq!(format!("{secret:?}"), "Secret(***)");
        assert_eq!(secret.expose(), "hunter2");
    }
}
