// External libraries
use actix_web::{App, HttpServer, middleware::Logger, web::Data};
use actix_files as fs;

// Internal modules
use crate::redis_helper::connection_manager::RedisClient;

#[actix_web::main]
pub async fn start_webserver(redis_client: RedisClient) -> std::io::Result<()> {
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080);

    let bind_address: String = std::env::var("BIND_ADDRESS")
        .ok()
        .unwrap_or("0.0.0.0".to_string());

    let logger_format = std::env::var("ACTIX_LOG_FORMAT").ok();


    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(redis_client.client.clone()).clone()) // Share Redis client across handlers
            .wrap(
                match &logger_format {
                    Some(format) => Logger::new(format), // Use custom log format if provided
                    None => Logger::default(), // Default logger
                }
            ) // Enable logging middleware
            .service(super::courses::get_courses)
            .service(super::lessons::get_lessons)
            .service(super::lessons_ics::get_ics_lessons)
            .service(
                fs::Files::new("/", "./src/public")
                    .index_file("index.html") // Serve index.html as the default file
                    .use_last_modified(true) // Use last modified header
                    .use_etag(true) // Use ETag for caching
            )
    })
    .bind((bind_address, port))?
    .run()
    .await
}
