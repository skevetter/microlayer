use anyhow::{Context, Result};
use log::error;
use std::fs;
use std::fs::File;
use std::path::PathBuf;

pub fn init_logging() -> Result<()> {
    let mut logger = env_logger::Builder::new();
    logger.parse_default_env();

    if let Ok(log_file_path) = std::env::var("PICOLAYER_LOG_FILE")
        && !log_file_path.is_empty()
    {
        let path = PathBuf::from(&log_file_path);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create log directory: {}", parent.display()))?;
        }

        logger.target(env_logger::Target::Pipe(Box::new(
            File::create(&path)
                .with_context(|| format!("Failed to open log file: {}", path.display()))?,
        )));
    }
    logger.target(env_logger::Target::Stderr);

    if let Ok(log_level) = std::env::var("PICOLAYER_LOG_LEVEL")
        && !log_level.is_empty()
    {
        if let Ok(level) = log_level.parse() {
            logger.filter_level(level);
        } else {
            error!("Invalid PICOLAYER_LOG_LEVEL: {}", log_level);
        }
    } else if let Ok(val) = std::env::var("RUST_LOG")
        && !val.is_empty()
    {
        if let Ok(level) = val.parse() {
            logger.filter_level(level);
        } else {
            error!("Invalid RUST_LOG: {}", val);
        }
    } else {
        logger.filter_level(log::LevelFilter::Warn);
    }

    logger.init();

    Ok(())
}
