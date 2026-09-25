//! Minimal RFC 5545 (iCalendar) writer for lessons.

mod format;

use chrono::{DateTime, Utc};

use self::format::{escape, format_time, line};
use crate::models::Lesson;

/// Suggested refresh interval for calendar clients.
const REFRESH: &str = "PT6H";

pub fn render(
    calendar_name: &str,
    timezone: &str,
    lessons: &[Lesson],
    now: DateTime<Utc>,
) -> String {
    let mut out = String::with_capacity(512 + lessons.len() * 400);

    line(&mut out, "BEGIN:VCALENDAR");
    line(&mut out, "VERSION:2.0");
    line(
        &mut out,
        "PRODID:-//TimeTable//github.com/jacopofilonzi/TimeTable//IT",
    );
    line(&mut out, "CALSCALE:GREGORIAN");
    line(&mut out, "METHOD:PUBLISH");
    line(&mut out, &format!("X-WR-CALNAME:{}", escape(calendar_name)));
    line(&mut out, &format!("X-WR-TIMEZONE:{timezone}"));
    line(
        &mut out,
        &format!("REFRESH-INTERVAL;VALUE=DURATION:{REFRESH}"),
    );
    line(&mut out, &format!("X-PUBLISHED-TTL:{REFRESH}"));

    let stamp = format_time(now);
    for lesson in lessons {
        event(&mut out, lesson, &stamp);
    }

    line(&mut out, "END:VCALENDAR");
    out
}

fn event(out: &mut String, lesson: &Lesson, stamp: &str) {
    line(out, "BEGIN:VEVENT");
    line(out, &format!("UID:{}@timetable", escape(&lesson.id)));
    line(out, &format!("DTSTAMP:{stamp}"));
    line(out, &format!("DTSTART:{}", format_time(lesson.start)));
    line(out, &format!("DTEND:{}", format_time(lesson.end)));
    line(out, &format!("SUMMARY:{}", escape(&lesson.subject)));
    if let Some(location) = &lesson.location {
        line(out, &format!("LOCATION:{}", escape(location)));
    }
    let description: Vec<&str> = [lesson.teacher.as_deref(), lesson.notes.as_deref()]
        .into_iter()
        .flatten()
        .collect();
    if !description.is_empty() {
        line(
            out,
            &format!("DESCRIPTION:{}", escape(&description.join("\n"))),
        );
    }
    line(out, "END:VEVENT");
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn renders_event() {
        let lesson = Lesson {
            id: "x-1".into(),
            start: Utc.with_ymd_and_hms(2026, 9, 22, 7, 30, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2026, 9, 22, 9, 30, 0).unwrap(),
            subject: "Analisi, parte 1".into(),
            teacher: Some("Rossi".into()),
            location: None,
            notes: Some("Aula cambiata".into()),
        };
        let ics = render("Test", "Europe/Rome", &[lesson], Utc::now());
        assert!(ics.contains("DTSTART:20260922T073000Z\r\n"));
        assert!(ics.contains("SUMMARY:Analisi\\, parte 1\r\n"));
        assert!(ics.contains("DESCRIPTION:Rossi\\nAula cambiata\r\n"));
        assert!(!ics.contains("LOCATION"));
    }
}
