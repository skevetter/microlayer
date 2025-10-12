use anyhow::{Context, Result};
use log::info;
use tempfile::TempDir;

use crate::{config, utils::filesystem};

fn brew_cmd() -> std::process::Command {
    std::process::Command::new("brew")
}

fn brew_install_packages(packages: &[String]) -> std::process::Command {
    let mut cmd = std::process::Command::new("brew");
    cmd.arg("install");
    for pkg in packages {
        cmd.arg(pkg);
    }
    cmd
}

fn brew_update() -> std::process::Command {
    let mut cmd = std::process::Command::new("brew");
    cmd.arg("update");
    cmd
}

fn brew_cleanup() -> std::process::Command {
    let mut cmd = std::process::Command::new("brew");
    cmd.arg("cleanup");
    cmd
}

fn brew_backup_cache(cache_backup: &std::path::Path) -> Result<()> {
    let cache_dir = match std::env::consts::OS {
        "macos" => dirs::home_dir()
            .context("Failed to get home directory")?
            .join("Library/Caches/Homebrew"),
        "linux" => dirs::home_dir()
            .context("Failed to get home directory")?
            .join(".cache/Homebrew"),
        _ => {
            return Ok(()); // Unsupported OS
        }
    };

    if !cache_dir.exists() {
        info!("No existing Homebrew cache to back up");
        return Ok(());
    }

    fs_extra::dir::copy(
        &cache_dir,
        cache_backup,
        &fs_extra::dir::CopyOptions::new().copy_inside(true),
    )
    .context("Failed to copy Homebrew cache")?;

    if let Ok(result) = filesystem::is_dissimilar_dirs(cache_backup, cache_dir)
        && result
    {
        anyhow::bail!("backup differs from original");
    }

    Ok(())
}

/// Install packages using Homebrew
pub fn install(packages: &[String]) -> Result<()> {
    let status = brew_cmd()
        .arg("--version")
        .status()
        .context("Failed to run 'brew --version'")?;

    if !status.success() {
        anyhow::bail!("Homebrew not found");
    }

    let temp_dir = TempDir::with_prefix(config::PICO_CONFIG.temp_dir_prefix)
        .context("Failed to create temp directory")?;
    let cache_backup = temp_dir.path().join("brew");

    info!("Backing up existing Homebrew cache");
    brew_backup_cache(&cache_backup).context("Failed to backup Homebrew cache")?;

    info!("Updating Homebrew");
    brew_update()
        .status()
        .context("Failed to update Homebrew")?;

    info!("Installing Homebrew packages: {:?}", packages);
    brew_install_packages(packages)
        .status()
        .context("Failed to install packages with Homebrew")?;

    info!("Cleaning up Homebrew cache");
    brew_cleanup()
        .status()
        .context("Failed to clean up Homebrew cache")?;

    if cache_backup.exists() {
        fs_extra::dir::copy(
            cache_backup,
            dirs::home_dir().unwrap().join("Library/Caches/Homebrew"),
            &fs_extra::dir::CopyOptions::new()
                .overwrite(true)
                .copy_inside(true),
        )
        .context("Failed to restore Homebrew cache")?;
    }

    Ok(())
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
