use log::info;
use redis::Client;

pub struct RedisClient {
    pub client: Client,
}

pub struct RedisCredentials {
    pub username: Option<String>,
    pub password: Option<String>,
    pub host: String,
    pub port: Option<u16>,
    pub db: Option<u8>,
}

impl RedisClient {
    pub fn new() -> redis::RedisResult<Self> {
        let connection_string = Self::get_connection_string();

        let client = Client::open(connection_string).map_err(|e| {
            eprintln!("Failed to create Redis client: {}", e);
            e
        })?;

        let _ = client.get_connection().map_err(|e| {
            eprintln!("Failed to get Redis connection: {}", e);
            e
        })?;

        info!("Redis connected successfully");
        Ok(Self { client })
    }

    fn get_connection_string() -> String {
        let credentials = RedisCredentials {
            username: std::env::var("REDIS_USER").ok().or(None),
            password: std::env::var("REDIS_PASSWD").ok().or(None),
            host: std::env::var("REDIS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: std::env::var("REDIS_PORT")
                .ok()
                .and_then(|p| p.parse().ok()),
            db: std::env::var("REDIS_DB").ok().and_then(|d| d.parse().ok()),
        };

        if credentials.username.is_some() && credentials.password.is_some() {
            format!(
                "redis://{}:{}@{}:{}/{}",
                credentials.username.as_ref().unwrap(),
                credentials.password.as_ref().unwrap(),
                credentials.host,
                credentials.port.unwrap_or(6379),
                credentials.db.unwrap_or(0)
            )
        } else {
            format!(
                "redis://{}:{}/{}",
                credentials.host,
                credentials.port.unwrap_or(6379),
                credentials.db.unwrap_or(0)
            )
        }
    }
}
