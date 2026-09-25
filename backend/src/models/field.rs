use serde::Serialize;

use super::{SelectOption, Text};

/// A wizard input; its `key` is the query parameter name.
#[derive(Debug, Clone, Serialize)]
pub struct Field {
    pub key: &'static str,
    pub label: Text,
    #[serde(flatten)]
    pub kind: FieldKind,
    /// Other fields whose values are needed to load this field's options.
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub depends_on: &'static [&'static str],
    /// Server-side sanity check applied to values of remote fields.
    #[serde(skip)]
    pub pattern: Option<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FieldKind {
    /// Options are known up-front and embedded in the schema.
    Select {
        options: Vec<SelectOption>,
        #[serde(skip_serializing_if = "Option::is_none")]
        default: Option<&'static str>,
    },
    /// Options must be fetched from `/api/universities/{id}/options/{key}`.
    RemoteSelect { searchable: bool },
}
