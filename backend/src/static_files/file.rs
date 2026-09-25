//! A single in-memory static file and its HTTP response (ETag, cache headers).

use std::{path::Path, sync::Arc};

use axum::{
    body::{Body, Bytes},
    http::{HeaderMap, HeaderValue, Method, StatusCode, header},
    response::Response,
};

use super::{BASE_PLACEHOLDER, fs::fnv1a};

/// Extensions whose content gets the base-path placeholder replaced.
const TEXT_EXTENSIONS: &[&str] = &[
    "html",
    "css",
    "js",
    "mjs",
    "json",
    "svg",
    "txt",
    "xml",
    "webmanifest",
    "map",
];

/// Astro's hashed assets: safe to cache forever.
const IMMUTABLE_PREFIX: &str = "_astro/";

pub struct StaticFile {
    body: Arc<[u8]>,
    content_type: String,
    etag: String,
    immutable: bool,
}

impl StaticFile {
    /// Reads `path` (served as `rel`), replacing the base-path placeholder in text files.
    pub fn load(path: &Path, rel: &str, base_path: &str) -> std::io::Result<Self> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let mut body = std::fs::read(path)?;
        if TEXT_EXTENSIONS.contains(&ext.as_str()) {
            body = match String::from_utf8(body) {
                Ok(text) => text.replace(BASE_PLACEHOLDER, base_path).into_bytes(),
                Err(err) => err.into_bytes(),
            };
        }
        let content_type = mime_guess::from_path(path)
            .first_raw()
            .map(|m| {
                if m.starts_with("text/") || m.ends_with("javascript") {
                    format!("{m}; charset=utf-8")
                } else {
                    m.to_string()
                }
            })
            .unwrap_or_else(|| "application/octet-stream".into());

        Ok(Self {
            etag: format!("\"{:016x}\"", fnv1a(&body)),
            body: body.into(),
            content_type,
            immutable: rel.starts_with(IMMUTABLE_PREFIX),
        })
    }

    /// Full response, or `304 Not Modified` when the client's `If-None-Match` matches.
    pub fn respond(&self, status: StatusCode, method: &Method, headers: &HeaderMap) -> Response {
        let cache_control = if self.immutable {
            "public, max-age=31536000, immutable"
        } else {
            "no-cache"
        };
        let not_modified = status == StatusCode::OK
            && headers
                .get(header::IF_NONE_MATCH)
                .and_then(|v| v.to_str().ok())
                .is_some_and(|v| v.split(',').any(|tag| tag.trim() == self.etag));

        let builder = Response::builder()
            .header(
                header::ETAG,
                HeaderValue::from_str(&self.etag).expect("etag is ascii"),
            )
            .header(header::CACHE_CONTROL, cache_control);

        let response = if not_modified {
            builder.status(StatusCode::NOT_MODIFIED).body(Body::empty())
        } else {
            let body = if method == Method::HEAD {
                Body::empty()
            } else {
                Body::from(Bytes::from_owner(ArcBytes(self.body.clone())))
            };
            builder
                .status(status)
                .header(header::CONTENT_TYPE, &self.content_type)
                .body(body)
        };
        response.expect("valid static response")
    }
}

/// Lets `Bytes` share the file's buffer without copying it.
struct ArcBytes(Arc<[u8]>);

impl AsRef<[u8]> for ArcBytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
