use crate::installers::apt_get;
use crate::utils::os;
use crate::{config, utils};
use anyhow::{Context, Result};
use log::debug;
use std::path::Path;
use tempfile::TempDir;

const APT_LISTS_DIR: &str = "/var/lib/apt/lists";

/// Install packages using aptitude
pub fn install(packages: &[String]) -> Result<()> {
    anyhow::ensure!(
        os::is_debian_like(),
        "aptitude should be used on Debian-like distributions (Debian, Ubuntu, etc.)"
    );

    utils::os::ensure_sudo().context("Failed to obtain sudo privileges")?;

    let temp_dir = TempDir::with_prefix(config::PICO_CONFIG.temp_dir_prefix)
        .context("Failed to create temp directory")?;
    let cache_backup = temp_dir.path().join("aptitude");
    debug!("Backup path {:?}", cache_backup);

    if os::copy_files(Path::new(APT_LISTS_DIR), &cache_backup).is_err() {
        anyhow::bail!("Failed to back up aptitude lists");
    }

    debug!("Updating aptitude repositories");
    aptitude_update()
        .output()
        .map(|o| debug!("Aptitude update output: {:?}", o))
        .context("Failed to update aptitude repositories")?;

    debug!("Installing aptitude");
    install_aptitude().context("Failed to install aptitude")?;

    debug!("Installing aptitude packages: {:?}", packages);
    aptitude_install(packages)
        .output()
        .map(|o| debug!("Aptitude install output: {:?}", o))
        .context("Failed to install packages with aptitude")?;

    debug!("Cleaning aptitude cache");
    aptitude_clean()
        .status()
        .context("Failed to clean aptitude cache")?;

    if os::copy_files(&cache_backup, Path::new(APT_LISTS_DIR)).is_err() {
        anyhow::bail!("Failed to restore aptitude lists from backup");
    }

    if temp_dir.path().exists() {
        anyhow::ensure!(
            temp_dir.close().is_ok(),
            "Failed to remove temporary directory"
        );
    } else {
        debug!("Temporary directory is deleted");
    }

    Ok(())
}

/// Installs the aptitude package using apt-get if not already installed
fn install_aptitude() -> Result<()> {
    if which::which("aptitude").is_err() {
        apt_get::install(&["aptitude".to_string()], Some(&[]), false)
            .context("Failed to install aptitude")?;
    }
    Ok(())
}

fn aptitude() -> std::process::Command {
    std::process::Command::new("aptitude")
}

fn aptitude_update() -> std::process::Command {
    let mut cmd = aptitude();
    cmd.arg("update");
    cmd
}

fn aptitude_clean() -> std::process::Command {
    let mut cmd = aptitude();
    cmd.arg("clean");
    cmd
}

fn aptitude_install(packages: &[String]) -> std::process::Command {
    let mut cmd = aptitude();
    cmd.arg("install")
        .arg("-y")
        .arg("--without-recommends")
        .args(packages);
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_aptitude() {
        let packages = vec!["curl".to_string()];
        let _ = aptitude_install(&packages);
    }
}
