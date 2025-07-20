// External libraries
use actix_web::{
    HttpResponse, Responder, get,
    http::StatusCode,
    web::{Data, Path, Query},
};
use log::error;
use redis::Client;
use serde_json::json;
use std::collections::HashMap;

// Internal modules
use crate::crawlers::store::get_university_crawler;
use crate::models::error::ErrorFault;

#[get("/timetable/{university}/courses")]
pub async fn get_courses(
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
        .get_cached_courses(university.clone(), query.into_inner(), redis_client)
        .await
    {
        Ok(courses) => {
            // Return the courses as JSON
            return HttpResponse::Ok().json(courses);
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
