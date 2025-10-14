mod cli;
mod installers;
mod utils;

use anyhow::{Context, Result};
use log::info;

fn main() -> Result<()> {
    utils::logging::init_logging().context("Failed to initialize logging")?;
    info!("Starting picolayer");
    cli::cli()?;

    Ok(())
}
