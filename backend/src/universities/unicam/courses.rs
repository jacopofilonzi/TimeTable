//! Course list, scraped from the timetable page's `<select id="selectPercorsi">`.

use std::sync::LazyLock;

use regex::Regex;

use super::html::decode_entities;
use crate::{
    errors::{AppError, Upstream},
    models::{SelectOption, Text},
};

const URL: &str = "https://orarilezioni.unicam.it/";

static OPTGROUP_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<optgroup\s+label="([^"]+)"[^>]*>([\s\S]*?)</optgroup>"#).unwrap()
});
static OPTION_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<option\b[^>]*\bvalue="(\d+)"[^>]*>([^<]+)</option>"#).unwrap());

pub async fn fetch(http: &reqwest::Client) -> Result<Vec<SelectOption>, AppError> {
    let html = http
        .get(URL)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| Upstream::new("unicam courses request", e))?
        .text()
        .await
        .map_err(|e| Upstream::new("unicam courses response", e))?;

    let courses = parse(&html);
    if courses.is_empty() {
        return Err(Upstream::msg("unicam courses page contained no courses").into());
    }
    Ok(courses)
}

/// `<optgroup label="School"><option value="ID">CODE - NAME</option>…` → options grouped by
/// school, with the code as hint.
fn parse(html: &str) -> Vec<SelectOption> {
    let mut courses = Vec::new();
    for group in OPTGROUP_RE.captures_iter(html) {
        let category = decode_entities(group[1].trim());
        for option in OPTION_RE.captures_iter(&group[2]) {
            let id = &option[1];
            if id == "0" {
                continue;
            }
            let text = decode_entities(option[2].trim());
            let (code, name) = match text.split_once(" - ") {
                Some((code, name)) => (Some(code.trim().to_string()), name.trim().to_string()),
                None => (None, text.clone()),
            };
            courses.push(SelectOption {
                value: id.to_string(),
                label: Text::plain(name),
                group: Some(category.clone()),
                hint: code,
            });
        }
    }
    courses
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_courses() {
        let html = r#"
            <optgroup label="Scuola di Architettura e Design">
                <option value="0">-</option>
                <option  value="10687">LM-AR - ARCHITETTURA</option>
                <option  value="10686">L-DIR - DESIGN</option>
            </optgroup>
            <optgroup label="Empty"></optgroup>"#;
        let courses = parse(html);
        assert_eq!(courses.len(), 2);
        assert_eq!(courses[0].value, "10687");
        assert_eq!(courses[0].hint.as_deref(), Some("LM-AR"));
        assert_eq!(
            courses[1].group.as_deref(),
            Some("Scuola di Architettura e Design")
        );
    }
}
