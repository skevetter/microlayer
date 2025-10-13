use crate::{config, utils::os};

use anyhow::{Context, Result};
use log::debug;
use tempfile::TempDir;

/// Install packages using Homebrew
pub fn install(packages: &[String]) -> Result<()> {
    anyhow::ensure!(
        which::which("brew").is_ok(),
        "Homebrew not installed or not in PATH"
    );

    let temp_dir = TempDir::with_prefix(config::PICO_CONFIG.temp_dir_prefix)
        .context("Failed to create temp directory")?;
    let cache_backup = temp_dir.path().join("brew");
    debug!("Backup path {:?}", cache_backup);

    let cache_dir = get_cache_dir().context("Failed to determine Homebrew cache directory")?;
    if os::copy_files(&cache_dir, &cache_backup).is_err() {
        anyhow::bail!("Failed to back up Homebrew cache");
    }
    debug!("Updating Homebrew");
    brew_update()
        .output()
        .map(|o| debug!("Brew update output: {:?}", o))
        .context("Failed to update Homebrew")?;

    debug!("Installing Homebrew packages: {:?}", packages);
    brew_install(packages)
        .output()
        .map(|o| debug!("Brew install output: {:?}", o))
        .context("Failed to install Homebrew packages")?;

    debug!("Cleaning up Homebrew cache");
    brew_cleanup()
        .output()
        .map(|o| debug!("Brew cleanup output: {:?}", o))
        .context("Failed to clean up Homebrew cache")?;

    if os::copy_files(&cache_backup, &cache_dir).is_err() {
        anyhow::bail!("Failed to restore Homebrew cache from backup");
    }
    if temp_dir.path().exists() {
        anyhow::ensure!(
            temp_dir.close().is_ok(),
            "Temporary directory could not be deleted"
        );
    } else {
        debug!("Temporary directory is deleted");
    }

    Ok(())
}

fn brew() -> std::process::Command {
    std::process::Command::new("brew")
}

fn brew_install(packages: &[String]) -> std::process::Command {
    let mut cmd = brew();
    cmd.arg("install").args(packages);
    cmd
}

fn brew_update() -> std::process::Command {
    let mut cmd = brew();
    cmd.arg("update");
    cmd
}

fn brew_cleanup() -> std::process::Command {
    let mut cmd = brew();
    cmd.arg("cleanup");
    cmd
}

fn get_cache_dir() -> Option<std::path::PathBuf> {
    match std::env::consts::OS {
        "macos" => dirs::home_dir().map(|d| d.join("Library/Caches/Homebrew")),
        "linux" => dirs::home_dir().map(|d| d.join(".cache/Homebrew")),
        _ => None, // Unsupported OS
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_install_function_exists() {
        let packages = vec!["nonexistent-package-12345".to_string()];
        let result = install(&packages);
        let _ = result;
    }
}
