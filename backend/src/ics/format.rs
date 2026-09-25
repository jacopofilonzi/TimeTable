//! Low-level iCalendar formatting: timestamps, text escaping, line folding.

use chrono::{DateTime, Utc};

/// Maximum content line length in octets (RFC 5545 §3.1).
const MAX_LINE_OCTETS: usize = 75;

/// UTC timestamp in the `YYYYMMDDTHHMMSSZ` form.
pub fn format_time(t: DateTime<Utc>) -> String {
    t.format("%Y%m%dT%H%M%SZ").to_string()
}

/// Escapes a TEXT value (RFC 5545 §3.3.11).
pub fn escape(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace(';', "\\;")
        .replace(',', "\\,")
        .replace("\r\n", "\\n")
        .replace('\n', "\\n")
        .replace('\r', "")
}

/// Writes a content line, folded at [`MAX_LINE_OCTETS`] without splitting UTF-8 characters.
pub fn line(out: &mut String, content: &str) {
    let mut width = 0;
    for ch in content.chars() {
        let len = ch.len_utf8();
        if width + len > MAX_LINE_OCTETS {
            out.push_str("\r\n ");
            width = 1;
        }
        out.push(ch);
        width += len;
    }
    out.push_str("\r\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folds_long_lines_on_char_boundaries() {
        let mut out = String::new();
        line(&mut out, &format!("SUMMARY:{}", "è".repeat(60)));
        for l in out.split("\r\n") {
            assert!(
                l.len() <= MAX_LINE_OCTETS,
                "line too long: {} octets",
                l.len()
            );
        }
        assert_eq!(
            out.replace("\r\n ", ""),
            format!("SUMMARY:{}\r\n", "è".repeat(60))
        );
    }

    #[test]
    fn escapes_text() {
        assert_eq!(escape("a,b;c\\d\ne"), "a\\,b\\;c\\\\d\\ne");
    }
}
