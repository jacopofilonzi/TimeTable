use std::{collections::HashMap, sync::Arc, time::Duration};

use super::AppState;
use crate::{
    errors::{AppError, NotFound},
    shortlinks::{ShortLinks, normalize},
    universities::{Params, University},
};

/// Codes never change once created, so lookups can stay cached for long.
const LOOKUP_TTL: Duration = Duration::from_secs(30 * 24 * 3600);

impl AppState {
    fn short_links(&self) -> Result<&ShortLinks, AppError> {
        self.short_links
            .as_ref()
            .ok_or_else(|| NotFound::new("Short links are disabled").into())
    }

    /// Validates `query` like the lessons endpoints and returns the short link code for it.
    pub async fn create_short_link(
        &self,
        uni: &Arc<dyn University>,
        query: &HashMap<String, String>,
    ) -> Result<String, AppError> {
        let links = self.short_links()?;
        let (params, weeks) = self.validate_lessons_query(uni, query).await?;
        links
            .create(page_query(uni.info().id, &params, weeks))
            .await
    }

    /// The page query (`uni=…&<fields>&weeks=N`) of a code as typed by a user, and records the
    /// visit.
    pub async fn resolve_short_link(&self, raw: &str) -> Result<String, AppError> {
        let links = self.short_links()?;
        let unknown = || AppError::from(NotFound::new("Unknown short link"));
        let code = normalize(raw).ok_or_else(unknown)?;
        let query = self
            .cache
            .get_or_fetch(&format!("short:{code}"), LOOKUP_TTL, || async {
                links.lookup(&code).await?.ok_or_else(unknown)
            })
            .await?;
        links.touch(&code).await;
        Ok((*query).clone())
    }
}

/// URL-encoded wizard page query for validated settings, in a fixed order so the same settings
/// always produce the same string (and so the same code).
fn page_query(uni: &str, params: &Params, weeks: u8) -> String {
    let mut query = form_urlencoded::Serializer::new(String::new());
    query.append_pair("uni", uni);
    for (key, value) in params {
        query.append_pair(key, value);
    }
    query.append_pair("weeks", &weeks.to_string());
    query.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_query_is_stable_and_encoded() {
        let params = Params::from([
            ("year".to_string(), "2".to_string()),
            ("course".to_string(), "A&B C".to_string()),
        ]);
        assert_eq!(
            page_query("unicam", &params, 4),
            "uni=unicam&course=A%26B+C&year=2&weeks=4"
        );
    }
}
