use serde::Serialize;

use super::{Step, Text};

/// A university and its wizard schema, as returned by `/api/universities`.
#[derive(Debug, Clone, Serialize)]
pub struct UniversityInfo {
    pub id: &'static str,
    pub name: Text,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<&'static str>,
    /// Wizard steps, in order. Each step groups one or more parameters on a single screen.
    pub steps: Vec<Step>,
    /// Allowed range for the `weeks` lessons parameter.
    pub weeks: WeeksRange,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct WeeksRange {
    pub min: u8,
    pub max: u8,
    pub default: u8,
}
