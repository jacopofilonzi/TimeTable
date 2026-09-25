use std::path::PathBuf;

use super::env;

/// Where the built frontend is looked for when `STATIC_DIR` is not set: the docker image path,
/// then the local build output (running from `backend/` or from the repo root).
const CANDIDATES: [&str; 3] = ["/www", "../frontend/dist", "frontend/dist"];

/// `STATIC_DIR` if set, otherwise the first candidate containing an `index.html`.
pub fn resolve() -> Option<PathBuf> {
    if let Some(dir) = env::var("STATIC_DIR") {
        return Some(PathBuf::from(dir));
    }
    CANDIDATES
        .into_iter()
        .map(PathBuf::from)
        .find(|p| p.join("index.html").is_file())
}
