use std::{collections::HashMap, sync::Arc};

use chrono::Utc;

use super::{
    AppState,
    validation::{all_fields, canonical, required, validate_shallow},
};
use crate::{
    errors::{AppError, BadRequest},
    models::{Field, FieldKind, Lesson, WeeksRange},
    universities::{DateRange, Params, University},
};

pub struct LessonsResult {
    pub lessons: Arc<Vec<Lesson>>,
    pub range: DateRange,
}

impl AppState {
    /// Validates `query` against the university's schema and returns the (cached) lessons.
    pub async fn lessons(
        &self,
        uni: &Arc<dyn University>,
        query: &HashMap<String, String>,
    ) -> Result<LessonsResult, AppError> {
        let info = uni.info();
        let fields: Vec<&Field> = all_fields(info).collect();

        let mut params = Params::new();
        for field in &fields {
            let value = required(query, field.key)?;
            validate_shallow(field, value)?;
            params.insert(field.key.to_string(), value.clone());
        }
        self.check_remote_values(uni, &fields, &params).await?;

        let weeks = parse_weeks(query.get("weeks"), info.weeks)?;
        let range = DateRange::current_weeks(Utc::now(), uni.timezone(), weeks);
        let key = format!(
            "lessons:{}:{}:{}:{weeks}",
            info.id,
            canonical(&params),
            range.from.date_naive()
        );
        let lessons = self
            .cache
            .get_or_fetch(&key, self.config.lessons_ttl, || {
                uni.lessons(&self.http, &params, &range)
            })
            .await?;
        Ok(LessonsResult { lessons, range })
    }

    /// Remote values must be among the (cached) options, so bogus values never reach the
    /// upstream site nor pollute the cache. If options can't be loaded, trust the pattern check.
    async fn check_remote_values(
        &self,
        uni: &Arc<dyn University>,
        fields: &[&Field],
        params: &Params,
    ) -> Result<(), AppError> {
        let remote = fields
            .iter()
            .filter(|f| matches!(f.kind, FieldKind::RemoteSelect { .. }));
        for field in remote {
            let deps: Params = field
                .depends_on
                .iter()
                .map(|d| (d.to_string(), params[*d].clone()))
                .collect();
            match self.remote_options(uni, field, &deps).await {
                Ok(options) if !options.iter().any(|o| o.value == params[field.key]) => {
                    return Err(
                        BadRequest::new(format!("Unknown value for '{}'", field.key)).into(),
                    );
                }
                Ok(_) => {}
                Err(err) => {
                    tracing::warn!(%err, field = field.key, "skipping option membership check")
                }
            }
        }
        Ok(())
    }
}

/// `weeks` query parameter: the university's default when missing, otherwise within its range.
fn parse_weeks(raw: Option<&String>, range: WeeksRange) -> Result<u8, AppError> {
    let Some(raw) = raw else {
        return Ok(range.default);
    };
    raw.parse::<u8>()
        .ok()
        .filter(|w| (range.min..=range.max).contains(w))
        .ok_or_else(|| {
            BadRequest::new(format!(
                "'weeks' must be between {} and {}",
                range.min, range.max
            ))
            .into()
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weeks_validation() {
        let range = WeeksRange {
            min: 1,
            max: 8,
            default: 4,
        };
        assert_eq!(parse_weeks(None, range).unwrap(), 4);
        assert_eq!(parse_weeks(Some(&"8".into()), range).unwrap(), 8);
        assert!(parse_weeks(Some(&"0".into()), range).is_err());
        assert!(parse_weeks(Some(&"9".into()), range).is_err());
        assert!(parse_weeks(Some(&"x".into()), range).is_err());
    }
}
