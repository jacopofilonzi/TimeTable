use serde::{Deserialize, Serialize};

use super::Text;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectOption {
    pub value: String,
    pub label: Text,
    /// Optional group heading (e.g. faculty / school).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// Optional secondary text (e.g. course code).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}
