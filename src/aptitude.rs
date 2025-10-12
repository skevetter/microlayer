use crate::apt_get;
use crate::utils::{filesystem, os};
use anyhow::{Context, Result};
use log::info;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

const APT_LISTS_DIR: &str = "/var/lib/apt/lists";

fn apt_get_install_aptitude() -> Result<()> {
    apt_get::install(&["aptitude".to_string()], Some(&[]), false)
        .context("Failed to install aptitude")?;
    Ok(())
}

fn apt_update() -> std::process::Command {
    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("aptitude").arg("update");
    cmd
}

fn apt_clean() -> std::process::Command {
    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("aptitude").arg("clean");
    cmd
}

fn apt_backup_lists(cache_backup: &Path) -> Result<()> {
    if !Path::new(APT_LISTS_DIR).exists() {
        info!("No existing apt lists cache to back up");
        return Ok(());
    }

    info!("Backing up apt lists cache to {:?}", cache_backup);
    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("cp")
        .arg("-p")
        .arg("-R")
        .arg(APT_LISTS_DIR)
        .arg(cache_backup.to_str().unwrap())
        .status()
        .context("Failed to copy apt lists cache")?;

    if let Ok(result) = filesystem::is_dissimilar_dirs(cache_backup, APT_LISTS_DIR)
        && result
    {
        anyhow::bail!("backup differs from original");
    }

    Ok(())
}

fn apt_rm_cache() -> std::process::Command {
    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("rm").arg("-rf").arg(APT_LISTS_DIR);
    cmd
}

fn aptitude_install_packages(packages: &[String]) -> std::process::Command {
    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("aptitude")
        .arg("install")
        .arg("-y")
        .arg("--without-recommends");
    for pkg in packages {
        cmd.arg(pkg);
    }
    cmd
}

/// Install packages using aptitude
pub fn install(packages: &[String]) -> Result<()> {
    anyhow::ensure!(
        os::is_debian_like(),
        "aptitude should be used on Debian-like distributions (Debian, Ubuntu, etc.)"
    );

    let temp_dir = TempDir::with_prefix("picolayer_").context("Failed to create temp directory")?;
    let cache_backup = temp_dir.path().join("aptitude");

    info!("Backing up existing apt lists cache");
    apt_backup_lists(&cache_backup)?;

    info!("Updating aptitude repositories");
    apt_update()
        .status()
        .context("Failed to update aptitude repositories")?;

    info!("Installing aptitude");
    apt_get_install_aptitude().context("Failed to install aptitude")?;

    info!("Installing aptitude packages: {:?}", packages);
    aptitude_install_packages(packages)
        .status()
        .context("Failed to install packages with aptitude")?;

    info!("Cleaning aptitude cache");
    apt_clean()
        .status()
        .context("Failed to clean aptitude cache")?;

    info!("Removing aptitude cache directory");
    apt_rm_cache()
        .status()
        .context("Failed to remove aptitude cache")?;

    if cache_backup.exists() {
        fs::rename(cache_backup, APT_LISTS_DIR).context("Failed to restore apt lists cache")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_aptitude() {
        let packages = vec!["curl".to_string()];
        let _ = aptitude_install_packages(&packages);
    }
}
