use serde::{Deserialize, Serialize};

/// Text that is either the same in every language or translated.
/// Serialized as a plain string or as `{ "it": ..., "en": ... }`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Text {
    Plain(String),
    Localized { it: String, en: String },
}

impl Text {
    pub fn plain(s: impl Into<String>) -> Self {
        Self::Plain(s.into())
    }

    pub fn localized(it: impl Into<String>, en: impl Into<String>) -> Self {
        Self::Localized {
            it: it.into(),
            en: en.into(),
        }
    }

    /// The text in the default language (Italian).
    pub fn default_lang(&self) -> &str {
        match self {
            Self::Plain(text) | Self::Localized { it: text, .. } => text,
        }
    }
}
