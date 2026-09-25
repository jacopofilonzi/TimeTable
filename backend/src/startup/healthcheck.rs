use std::time::Duration;

use crate::config::Config;

const TIMEOUT: Duration = Duration::from_secs(3);

/// `timetable healthcheck`: used by the Docker HEALTHCHECK (the runtime image has no curl).
/// Returns the process exit code: 0 if `/api/health` answers successfully, 1 otherwise.
pub async fn run_healthcheck(config: &Config) -> i32 {
    let url = format!(
        "http://127.0.0.1:{}{}/api/health",
        config.port, config.base_path
    );
    let ok = reqwest::Client::new()
        .get(&url)
        .timeout(TIMEOUT)
        .send()
        .await
        .is_ok_and(|r| r.status().is_success());
    if ok { 0 } else { 1 }
}
