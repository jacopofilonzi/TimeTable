//! Lessons, from the JSON endpoint used by the timetable page's calendar.

use chrono::{DateTime, SecondsFormat, Utc};
use serde::Deserialize;

use super::html::{decode_entities, parse_description};
use crate::{
    errors::{AppError, Upstream},
    models::Lesson,
    universities::DateRange,
};

const URL: &str = "https://unifare.unicam.it/controller/ajaxController.php";

pub async fn fetch(
    http: &reqwest::Client,
    course_id: &str,
    course_year: &str,
    range: &DateRange,
) -> Result<Vec<Lesson>, AppError> {
    let from = range.from.to_rfc3339_opts(SecondsFormat::Secs, false);
    let to = range.to.to_rfc3339_opts(SecondsFormat::Secs, false);

    let raw: Vec<RawLesson> = http
        .get(URL)
        .query(&[
            ("filename", "../didattica/controller/orari.php"),
            ("class", "OrariController"),
            ("method", "getDateLezioniByPercorsoCalendar"),
            ("parametri[]", course_id),
            ("parametri[]", "false"),
            ("parametri[]", course_year),
            ("start", &from),
            ("end", &to),
        ])
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| Upstream::new("unicam lessons request", e))?
        .json()
        .await
        .map_err(|e| Upstream::new("unicam lessons response", e))?;

    Ok(raw.into_iter().filter_map(RawLesson::into_lesson).collect())
}

/// A lesson as sent by unicam (only the fields we use).
#[derive(Deserialize)]
struct RawLesson {
    id: serde_json::Value,
    title: Option<String>,
    description: Option<String>,
    start: serde_json::Value,
    end: serde_json::Value,
}

impl RawLesson {
    /// `None` (logged) when start/end can't be parsed.
    fn into_lesson(self) -> Option<Lesson> {
        let (start, end) = match (parse_time(&self.start), parse_time(&self.end)) {
            (Some(start), Some(end)) => (start, end),
            _ => {
                tracing::warn!(id = %self.id, "skipping unicam lesson with invalid start/end");
                return None;
            }
        };
        let details = self
            .description
            .as_deref()
            .map(parse_description)
            .unwrap_or_default();
        let id = self
            .id
            .as_str()
            .map_or_else(|| self.id.to_string(), str::to_string);
        Some(Lesson {
            id: format!("unicam-{id}"),
            start,
            end,
            subject: decode_entities(self.title.as_deref().unwrap_or("").trim()),
            teacher: details.teacher,
            location: details.location,
            notes: details.notes,
        })
    }
}

/// Unicam sends epoch milliseconds; accept numeric strings and RFC 3339 too, just in case.
fn parse_time(value: &serde_json::Value) -> Option<DateTime<Utc>> {
    match value {
        serde_json::Value::Number(n) => DateTime::from_timestamp_millis(n.as_i64()?),
        serde_json::Value::String(s) => match s.parse::<i64>() {
            Ok(ms) => DateTime::from_timestamp_millis(ms),
            Err(_) => DateTime::parse_from_rfc3339(s)
                .ok()
                .map(|d| d.with_timezone(&Utc)),
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_raw_lesson() {
        let raw: RawLesson = serde_json::from_str(
            r#"{"id":451175,"title":"LAB 1","description":"Aula 1 <div></div><b>Docenti:</b> X","start":1790062200000,"end":1790074800000}"#,
        )
        .unwrap();
        let lesson = raw.into_lesson().unwrap();
        assert_eq!(lesson.id, "unicam-451175");
        assert_eq!(lesson.start.to_rfc3339(), "2026-09-22T07:30:00+00:00");
        assert_eq!(lesson.teacher.as_deref(), Some("X"));
    }

    #[test]
    fn parses_time_formats() {
        let expected = "2026-09-22T07:30:00+00:00";
        for value in [
            serde_json::json!(1790062200000_i64),
            serde_json::json!("1790062200000"),
            serde_json::json!("2026-09-22T09:30:00+02:00"),
        ] {
            assert_eq!(parse_time(&value).unwrap().to_rfc3339(), expected);
        }
        assert!(parse_time(&serde_json::json!(null)).is_none());
    }
}
