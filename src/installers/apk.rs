use crate::config;
use crate::utils::os;
use anyhow::{Context, Result};
use log::debug;
use std::path::Path;
use tempfile::TempDir;

const APK_CACHE_DIR: &str = "/var/cache/apk";

/// Install packages using apk
pub fn install(packages: &[String]) -> Result<()> {
    anyhow::ensure!(
        os::is_alpine(),
        "apk should be used on Alpine Linux distribution"
    );

    let temp_dir = TempDir::with_prefix(config::PICO_CONFIG.temp_dir_prefix)
        .context("Failed to create temp directory")?;
    let cache_backup = temp_dir.path().join("apk");
    debug!("Backup path {:?}", cache_backup);

    os::copy_files(Path::new(APK_CACHE_DIR), &cache_backup).context("Failed to copy apk cache")?;

    debug!("Updating apk repositories");
    apk_update()
        .output()
        .map(|o| debug!("Apk update output: {:?}", o))
        .context("Failed to update apk repositories")?;

    debug!("Installing apk packages: {:?}", packages);
    apk_add_pkgs(packages)
        .output()
        .map(|o| debug!("Apk add output: {:?}", o))
        .context("Failed to install apk packages")?;

    debug!("Cleaning up apk cache");
    apk_clean()
        .output()
        .map(|o| debug!("Apk clean output: {:?}", o))
        .context("Failed to clean apk cache")?;

    os::copy_files(&cache_backup, Path::new(APK_CACHE_DIR))
        .context("Failed to restore apk cache")?;

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

fn apk() -> std::process::Command {
    std::process::Command::new("apk")
}

fn apk_clean() -> std::process::Command {
    let mut cmd = apk();
    cmd.arg("cache").arg("clean");
    cmd
}

fn apk_update() -> std::process::Command {
    let mut cmd = apk();
    cmd.arg("update");
    cmd
}

fn apk_add_pkgs(packages: &[String]) -> std::process::Command {
    let mut cmd = apk();
    cmd.arg("add").arg("--no-cache").args(packages);
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_apk() {
        let packages = vec!["test".to_string()];
        let result = install(&packages);
        let _ = result;
    }
}
