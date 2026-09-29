//! Route handlers, one module per endpoint group.

mod admin;
mod health;
mod lessons;
mod lessons_ics;
mod options;
mod short_links;
mod universities;

pub use admin::clear_cache;
pub use health::health;
pub use lessons::get_lessons;
pub use lessons_ics::get_lessons_ics;
pub use options::get_options;
pub use short_links::{create_short_link, open_short_link};
pub use universities::{get_university, list_universities};
