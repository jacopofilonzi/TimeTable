use crate::{config::Config, static_files::StaticFiles};

/// Loads the built frontend, or serves the API only when no build was found.
pub fn load_frontend(config: &Config) -> std::io::Result<StaticFiles> {
    let Some(dir) = &config.static_dir else {
        tracing::warn!(
            "frontend build not found (set STATIC_DIR or build the frontend), serving API only"
        );
        return Ok(StaticFiles::default());
    };
    let files = StaticFiles::load(dir, &config.base_path)?;
    tracing::info!(
        "serving {} frontend files from {}",
        files.len(),
        dir.display()
    );
    Ok(files)
}
