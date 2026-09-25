//! University crawlers. Each one describes its wizard steps/parameters and knows how to fetch
//! options and lessons from its own website.
//!
//! To add a university: implement [`University`] in a new module and register it in
//! [`Registry::new`].

mod date_range;
mod registry;
mod university;

mod unicam;

pub use date_range::DateRange;
pub use registry::Registry;
pub use university::{Params, University};
