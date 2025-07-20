// External libraries
use dotenv::dotenv;
use log::info;

// Initialize crates
mod api;
mod crawlers;
mod models;
mod redis_helper;

fn main() {
    dotenv().ok(); // Load environment variables from .env file
    env_logger::init(); // Initialize the logger

    info!("╔══════════════════════════════════╗");
    info!("║            TimeTable             ║");
    info!("║     github.com/jacopofilonzi     ║");
    info!("╚══════════════════════════════════╝");

    let redis_client = redis_helper::connection_manager::RedisClient::new()
        .expect("Failed to create Redis connection manager");

    // test::main();
    let _ = api::main::start_webserver(redis_client); // Start the API server
}
