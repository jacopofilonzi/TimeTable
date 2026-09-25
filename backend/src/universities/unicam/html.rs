//! HTML helpers for unicam's markup.

use std::sync::LazyLock;

use regex::Regex;

static LINE_BREAK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)<br\s*/?>|</?div[^>]*>").unwrap());
static TAG_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[^>]+>").unwrap());
static SPACES_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[ \t]{2,}").unwrap());

/// What unicam packs into a lesson's `description`.
#[derive(Default, Debug, PartialEq)]
pub struct Details {
    pub location: Option<String>,
    pub teacher: Option<String>,
    pub notes: Option<String>,
}

/// Parses descriptions like
/// `Aula A <div style="height:8px"></div><b>Docenti:</b> ROSSI MARIO<br><i><b>Avvisi:</b> ...</i>`.
pub fn parse_description(html: &str) -> Details {
    let text = LINE_BREAK_RE.replace_all(html, "\n");
    let text = decode_entities(&TAG_RE.replace_all(&text, ""));

    let mut details = Details::default();
    let mut location = Vec::new();
    for line in text.lines().map(|l| SPACES_RE.replace_all(l.trim(), " ")) {
        if line.is_empty() {
            continue;
        }
        if let Some(teacher) = line.strip_prefix("Docenti:") {
            details.teacher = non_empty(teacher);
        } else if let Some(notes) = line.strip_prefix("Avvisi:") {
            details.notes = non_empty(notes);
        } else {
            location.push(line.into_owned());
        }
    }
    details.location = non_empty(&location.join(", "));
    details
}

/// Decodes the few HTML entities unicam uses.
pub fn decode_entities(s: &str) -> String {
    s.replace("&quot;", "\"")
        .replace("&#039;", "'")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
}

fn non_empty(s: &str) -> Option<String> {
    let s = s.trim();
    (!s.is_empty()).then(|| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_description() {
        let d = parse_description(
            r#"Sad Polo Sant'angelo Magno  - Laboratorio C135 <div style="height:8px"></div><b>Docenti:</b> GARAGUSO ANTONELLO<br><i><b>Avvisi:</b> Prolusione congiunta</i>"#,
        );
        assert_eq!(
            d,
            Details {
                location: Some("Sad Polo Sant'angelo Magno - Laboratorio C135".into()),
                teacher: Some("GARAGUSO ANTONELLO".into()),
                notes: Some("Prolusione congiunta".into()),
            }
        );
    }

    #[test]
    fn parses_description_without_teacher() {
        let d = parse_description("Aula Magna");
        assert_eq!(d.location.as_deref(), Some("Aula Magna"));
        assert_eq!(d.teacher, None);
    }

    #[test]
    fn decodes_entities() {
        assert_eq!(decode_entities("L&#039;Aquila &amp; Co"), "L'Aquila & Co");
    }
}
