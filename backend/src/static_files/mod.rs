//! Serves the built frontend from memory.
//!
//! The frontend is built with the placeholder base path [`BASE_PLACEHOLDER`]; at startup every
//! text file is loaded and the placeholder is replaced with the configured `BASE_PATH`, so a single
//! build (and docker image) works under any prefix.

mod file;
mod fs;

use std::{collections::HashMap, path::Path, sync::Arc};

use axum::{
    http::{HeaderMap, Method, StatusCode, Uri},
    response::{IntoResponse, Response},
};

use self::file::StaticFile;

/// Must match `base` in `frontend/astro.config.mjs` for production builds.
pub const BASE_PLACEHOLDER: &str = "/__TT_BASE__";

#[derive(Clone, Default)]
pub struct StaticFiles {
    /// Keyed by path relative to the build dir, with `/` separators (e.g. `en/index.html`).
    files: Arc<HashMap<String, StaticFile>>,
}

impl StaticFiles {
    pub fn load(dir: &Path, base_path: &str) -> std::io::Result<Self> {
        let mut files = HashMap::new();
        for path in fs::walk(dir)? {
            let rel = path
                .strip_prefix(dir)
                .expect("walked path is inside dir")
                .to_string_lossy()
                .replace('\\', "/");
            let file = StaticFile::load(&path, &rel, base_path)?;
            files.insert(rel, file);
        }
        Ok(Self {
            files: Arc::new(files),
        })
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Resolves `/`, `/en`, `/en/` to `index.html` files and serves `404.html` for misses.
    pub fn respond(&self, method: &Method, uri: &Uri, headers: &HeaderMap) -> Response {
        if method != Method::GET && method != Method::HEAD {
            return StatusCode::METHOD_NOT_ALLOWED.into_response();
        }
        let path = uri.path().trim_start_matches('/');
        let candidates = [
            path.to_string(),
            format!("{}/index.html", path.trim_end_matches('/'))
                .trim_start_matches('/')
                .to_string(),
        ];
        let found = candidates
            .iter()
            .find_map(|c| self.files.get(c.as_str()).filter(|_| !c.is_empty()));

        match found {
            Some(file) => file.respond(StatusCode::OK, method, headers),
            None => match self.files.get("404.html") {
                Some(file) => file.respond(StatusCode::NOT_FOUND, method, headers),
                None => StatusCode::NOT_FOUND.into_response(),
            },
        }
    }
}
