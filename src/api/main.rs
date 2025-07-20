// External libraries
use actix_web::{App, HttpServer, middleware::Logger, web::Data};

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

    // let redis_data = Data::new(redis_client.client);

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(redis_client.client.clone()).clone()) // Share Redis client across handlers
            .wrap(Logger::default()) // Enable logging middleware
            .service(super::courses::get_courses)
            .service(super::lessons::get_lessons)
            .service(super::lessons_ics::get_ics_lessons)
    })
    .bind((bind_address, port))?
    .run()
    .await
}
