use std::collections::BTreeMap;

use async_trait::async_trait;
use chrono_tz::Tz;

use super::DateRange;
use crate::{
    errors::AppError,
    models::{Lesson, SelectOption, UniversityInfo},
};

/// Validated parameters, sorted by key (so they can be used to build stable cache keys).
pub type Params = BTreeMap<String, String>;

/// A university crawler.
#[async_trait]
pub trait University: Send + Sync {
    /// Identity and wizard schema.
    fn info(&self) -> &UniversityInfo;

    /// Timezone used to compute week boundaries.
    fn timezone(&self) -> Tz {
        chrono_tz::Europe::Rome
    }

    /// Options for a `remote_select` field. `params` contains only the field's validated
    /// dependencies.
    async fn options(
        &self,
        http: &reqwest::Client,
        field: &str,
        params: &Params,
    ) -> Result<Vec<SelectOption>, AppError>;

    /// Lessons in `range`. `params` contains every field of the schema, validated.
    async fn lessons(
        &self,
        http: &reqwest::Client,
        params: &Params,
        range: &DateRange,
    ) -> Result<Vec<Lesson>, AppError>;
}
