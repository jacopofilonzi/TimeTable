use serde::Serialize;

use super::{Field, Text};

/// One wizard screen.
#[derive(Debug, Clone, Serialize)]
pub struct Step {
    pub id: &'static str,
    pub title: Text,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Text>,
    pub fields: Vec<Field>,
}
