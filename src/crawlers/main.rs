// External libraries
use actix_web::web::Data;
use async_trait::async_trait;
use log::warn;
use md5;
use redis::{Client, Commands, RedisResult};
use std::collections::HashMap;

// Internal modules
use crate::models::{course::Course, error::Error, lesson::Lesson};

// This trait is the common interface for all crawlers
#[async_trait]
pub trait UniversityCrawler: Send + Sync {
    // To be implemented by each crawler

    /// Fetches lessons based on the provided query parameters.
    async fn get_lessons(&self, query: HashMap<String, String>) -> Result<Vec<Lesson>, Error>;
    /// Fetches courses based on the provided query parameters.
    async fn get_courses(&self, query: HashMap<String, String>) -> Result<Vec<Course>, Error>;

    // ================ Caching methods =================
    // This methods are common for all crawlers to implement caching by hashing query paramethers of the request
    // and shouldn't be overridden by the crawlers

    /// Returns cached lessons
    async fn get_cached_lessons(
        &self,
        university: String,
        query: HashMap<String, String>,
        redis_client: Data<Client>,
    ) -> Result<Vec<Lesson>, Error> {
        // Get a connection to Redis
        let mut redis_conn = match redis_client.get_connection() {
            Ok(conn) => conn, //OK
            Err(err) => {
                // Connection failed, skip cache and crawl
                warn!("Failed to connect to Redis: {}", err);
                return self.get_lessons(query).await;
            }
        };

        // Sort the query parameters to ensure consistent hashing regardless of order
        let mut sorted_query: Vec<(&String, &String)> = query.iter().collect();
        sorted_query.sort_by_key(|&(k, _)| k);

        // Create a hash of the query parameters to use as a key
        let query_hash = format!(
            "{:x}",
            md5::compute(serde_json::to_string(&sorted_query).unwrap())
        );

        // Fetch cache
        let cache_result: RedisResult<String> =
            redis_conn.get(format!("lessons:{}:{}", university, query_hash));

        match cache_result {
            Ok(lessons) => {
                // Cache hit
                return Ok(serde_json::from_str::<Vec<Lesson>>(&lessons).unwrap());
            }
            Err(_) => {
                // Cache miss, fetch from crawler
                // Crawl
                let _lessons = self.get_lessons(query).await?;

                // Cache the courses for 3 days || key -> `lessons:<university>:<sorted_query_hash>`
                let _: RedisResult<()> = redis_conn.set_ex(
                    format!("lessons:{}:{}", university, query_hash),
                    serde_json::to_string(&_lessons).unwrap(),
                    60 * 60 * 24 * 3,
                );

                // Return the courses fetched from the crawler
                return Ok(_lessons);
            }
        }
    }

    // -----------------------------------------------------------------------------------------------------------------------

    /// Returns cached courses
    async fn get_cached_courses(
        &self,
        university: String,
        query: HashMap<String, String>,
        redis_client: Data<Client>,
    ) -> Result<Vec<Course>, Error> {
        // Get a connection to Redis
        let mut redis_conn = match redis_client.get_connection() {
            Ok(conn) => conn, //OK
            Err(err) => {
                // Connection failed, skip cache and crawl
                warn!("Failed to connect to Redis: {}", err);
                return self.get_courses(query).await;
            }
        };

        // Sort the query parameters to ensure consistent hashing regardless of order
        let mut sorted_query: Vec<(&String, &String)> = query.iter().collect();
        sorted_query.sort_by_key(|&(k, _)| k);

        // Create a hash of the query parameters to use as a key
        let query_hash = format!(
            "{:x}",
            md5::compute(serde_json::to_string(&sorted_query).unwrap())
        );

        // Fetch cache
        let cache_result: RedisResult<String> =
            redis_conn.get(format!("courses:{}:{}", university, query_hash));

        match cache_result {
            Ok(courses) => {
                // Cache hit
                return Ok(serde_json::from_str::<Vec<Course>>(&courses).unwrap());
            }
            Err(_) => {
                // Cache miss, fetch from crawler
                // Crawl
                let _courses = self.get_courses(query).await?;

                // Cache the courses for 3 months || key -> `courses:<university>:<sorted_query_hash>`
                let _: RedisResult<()> = redis_conn.set_ex(
                    format!("courses:{}:{}", university, query_hash),
                    serde_json::to_string(&_courses).unwrap(),
                    60 * 60 * 24 * 90,
                );

                // Return the courses fetched from the crawler
                return Ok(_courses);
            }
        }
    }
}
