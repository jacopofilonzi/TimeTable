// External libraries
use actix_web::{
    HttpResponse, Responder, get,
    http::StatusCode,
    web::{Data, Path, Query},
};
use chrono::{TimeZone, Utc};
use log::error;
use redis::Client;
use serde_json::json;
use std::collections::HashMap;

// Internal modules
use crate::models::error::ErrorFault;
use crate::{crawlers::store::get_university_crawler, models::lesson::Lesson};

#[get("/timetable/{university}/lessons.ics")]
pub async fn get_ics_lessons(
    path: Path<String>,
    query: Query<HashMap<String, String>>,
    redis_client: Data<Client>,
) -> impl Responder {
    // Extract the university name from the path and convert it to lowercase
    let university = path.into_inner().to_lowercase().trim().to_string();

    // Find crawler
    let crawler = match get_university_crawler(&university) {
        Some(crawler) => crawler,
        None => {
            // Crawler not found -> 404 Not Found
            return HttpResponse::NotFound()
                .body(json!({"error": "Not Found", "message": format!("No crawler found for university '{}', you can make your proposal at https://github.com/jacopofilonzi/timetable", university)}).to_string());
        }
    };

    match crawler
        .get_cached_lessons(university.clone(), query.into_inner(), redis_client)
        .await
    {
        Ok(lessons) => {
            // Return the courses as JSON
            // return HttpResponse::Ok().json(lessons);
            return HttpResponse::build(StatusCode::OK)
                .append_header(("Content-Disposition", "attachment; filename=timetable.ics"))
                .append_header(("Content-Type", "text/calendar"))
                .body(format_lessons_to_ics(&lessons, &university));
        }

        Err(error) => {
            // An error occurred while getting courses
            match error.fault {
                ErrorFault::User => {
                    return HttpResponse::build(
                        StatusCode::from_u16(error.http_code.unwrap_or(400)).unwrap(),
                    )
                    .body(json!({ "error": error.error, "message": error.message }).to_string());
                }

                ErrorFault::Internal => {
                    error!(
                        "Internal error:\nCrawler: {:#?} \n{:#?}\n{:#?}\n{:#?}",
                        university,
                        error.error,
                        error.fault,
                        error.message.unwrap_or("<no message>".to_string())
                    );

                    return HttpResponse::build(StatusCode::from_u16(500).unwrap())
                    .body(json!({"error": "Internal ServerError", "message":"An internal error occurred"}).to_string());
                }

                ErrorFault::External => {
                    error!(
                        "External error:\nCrawler: {:#?} \n{:#?}\n{:#?}\n{:#?}",
                        university,
                        error.error,
                        error.fault,
                        error.message.unwrap_or("<no message>".to_string())
                    );

                    return HttpResponse::build(StatusCode::from_u16(502).unwrap())
                    .body(json!({"error": "Bad Gateway", "message": "An external service error occurred"}).to_string());
                }
            }
        }
    };
}

fn format_lessons_to_ics(lessons: &Vec<Lesson>, university: &String) -> String {
    let now = Utc::now();

    let mut ics = String::new();

    // ICS Header
    ics.push_str("BEGIN:VCALENDAR\r\n");
    ics.push_str("VERSION:2.0\r\n");
    ics.push_str("PRODID:-//Timetable//Timetable Calendar//IT\r\n");
    ics.push_str("CALSCALE:GREGORIAN\r\n");
    ics.push_str(&format!(
        "X-WR-CALNAME:{} Timetable\r\n",
        escape_ics_text(&university.to_uppercase())
    ));
    ics.push_str(&format!(
        "X-WR-CALDESC:Lessons timetable for {}\r\n",
        escape_ics_text(&university)
    ));

    for lesson in lessons {
        // Create a single event with course information
        ics.push_str("BEGIN:VEVENT\r\n");
        ics.push_str(&format!(
            "UID:timetable-{}-{}-{}\r\n",
            escape_ics_text(&university),
            escape_ics_text(&lesson.subject),
            escape_ics_text(&lesson.starts_at)
        ));
        ics.push_str(&format!(
            "DTSTAMP:{}\r\n",
            timestamp_to_utc_datetime(now.timestamp_millis())
        ));
        ics.push_str(&format!(
            "DTSTART:{}\r\n",
            timestamp_to_utc_datetime(lesson.starts_at.parse::<i64>().unwrap_or(0))
        ));
        ics.push_str(&format!(
            "DTEND:{}\r\n",
            timestamp_to_utc_datetime(lesson.ends_at.parse::<i64>().unwrap_or(0))
        ));
        ics.push_str(&format!("SUMMARY:{}\r\n", escape_ics_text(&lesson.subject)));
        ics.push_str(&format!(
            "DESCRIPTION:{}\r\n",
            escape_ics_text(&lesson.description.as_deref().unwrap_or(""))
        ));
        ics.push_str(&format!(
            "LOCATION:{}\r\n",
            escape_ics_text(&lesson.location.as_deref().unwrap_or(""))
        ));
        ics.push_str("END:VEVENT\r\n");
    }

    // ICS Footer
    ics.push_str("END:VCALENDAR\r\n");

    ics
}

/// Escapes special characters in ICS text fields
fn escape_ics_text(text: &str) -> String {
    text.replace("\\", "\\\\")
        .replace(",", "\\,")
        .replace(";", "\\;")
        .replace("\n", "\\n")
        .replace("\r", "")
}

/// Converts a millisecond timestamp to ICS format (YYYYMMDDTHHMMSSZ)
pub fn timestamp_to_utc_datetime(timestamp_millis: i64) -> String {
    let datetime = Utc
        .timestamp_millis_opt(timestamp_millis)
        .single()
        .expect("Invalid timestamp");

    datetime.format("%Y%m%dT%H%M%SZ").to_string()
}
