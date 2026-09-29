use crate::{config::Config, shortlinks::ShortLinks};

/// Opens the short link database, or `None` when disabled. Fails (and so stops the server) when
/// the configured file can't be created or written.
pub fn open_short_links(config: &Config) -> Result<Option<ShortLinks>, String> {
    let Some(path) = &config.shortlink_db else {
        tracing::info!("short links disabled (SHORTLINK_DB=false)");
        return Ok(None);
    };
    let links = ShortLinks::open(path).map_err(|e| {
        format!(
            "cannot open the short link database {} (set SHORTLINK_DB to a writable path, or \
             SHORTLINK_DB=false to disable short links): {e}",
            path.display()
        )
    })?;
    tracing::info!("short links stored in {}", path.display());
    Ok(Some(links))
}
