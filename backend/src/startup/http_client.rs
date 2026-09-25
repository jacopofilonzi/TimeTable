use std::time::Duration;

const USER_AGENT: &str = concat!(
    "TimeTable/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/jacopofilonzi/TimeTable)"
);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// The HTTP client shared by all crawlers.
pub fn http_client() -> reqwest::Result<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(REQUEST_TIMEOUT)
        .build()
}
