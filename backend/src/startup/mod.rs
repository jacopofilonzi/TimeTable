//! Process-level setup used by `main`: logging, HTTP client, frontend loading, healthcheck
//! subcommand, graceful shutdown.

mod frontend;
mod healthcheck;
mod http_client;
mod logging;
mod shutdown;

pub use frontend::load_frontend;
pub use healthcheck::run_healthcheck;
pub use http_client::http_client;
pub use logging::init_logging;
pub use shutdown::shutdown_signal;
