//! Route handlers, one module per endpoint group.

mod admin;
mod health;
mod lessons;
mod lessons_ics;
mod options;
mod short_links;
mod stats;
mod universities;

use axum::{http::StatusCode, response::Response};

pub use admin::clear_cache;
pub use health::health;
pub use lessons::get_lessons;
pub use lessons_ics::get_lessons_ics;
pub use options::get_options;
pub use short_links::{create_short_link, open_short_link};
pub use stats::{metrics, stats_requests, stats_subscribers};
pub use universities::{get_university, list_universities};

use crate::errors::AppError;

/// Status a handler result is answered with, for tracking.
fn status_of(result: &Result<Response, AppError>) -> StatusCode {
    match result {
        Ok(response) => response.status(),
        Err(err) => err.status(),
    }
}
