//! Process-level setup used by `main`: `.env`, logging, HTTP client, frontend loading, short link
//! database, healthcheck subcommand, graceful shutdown.

mod env_file;
mod frontend;
mod healthcheck;
mod http_client;
mod logging;
mod short_links;
mod shutdown;

pub use env_file::load_env_file;
pub use frontend::load_frontend;
pub use healthcheck::run_healthcheck;
pub use http_client::http_client;
pub use logging::init_logging;
pub use short_links::open_short_links;
pub use shutdown::shutdown_signal;
