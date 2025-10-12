use crate::config;
use crate::utils::{filesystem, os};
use anyhow::{Context, Result};
use log::info;
use std::path::Path;
use tempfile::TempDir;

const APK_CACHE_DIR: &str = "/var/cache/apk";

fn apk_backup_cache(cache_backup: &Path) -> Result<()> {
    if !Path::new(APK_CACHE_DIR).exists() {
        info!("No existing apk cache to back up");
        return Ok(());
    }

    info!("Backing up apk cache to {:?}", cache_backup);
    match fs_extra::dir::copy(
        APK_CACHE_DIR,
        cache_backup,
        &fs_extra::dir::CopyOptions::new(),
    ) {
        Ok(_) => {}
        Err(err) if matches!(err.kind, fs_extra::error::ErrorKind::PermissionDenied) => {
            info!("Restoring apk cache with sudo");
            std::process::Command::new("sudo")
                .arg("cp")
                .arg("-r")
                .arg(cache_backup)
                .arg(APK_CACHE_DIR)
                .status()
                .context("Failed to restore apk cache with sudo")?;
        }
        Err(err) => {
            return Err(anyhow::anyhow!("Failed to restore apk cache: {}", err));
        }
    }

    match filesystem::is_dissimilar_dirs(cache_backup, APK_CACHE_DIR) {
        Ok(is_different) => {
            if is_different {
                anyhow::bail!("backup differs from original");
            }
            info!("Cache backup and source are the same!")
        }
        Err(err) if matches!(err.kind(), Some(std::io::ErrorKind::PermissionDenied)) => {
            info!("Comparing apt lists with sudo");
            let status = std::process::Command::new("sudo")
                .arg("diff")
                .arg("-r")
                .arg(APK_CACHE_DIR)
                .arg(cache_backup)
                .status()
                .context("Failed to compare apk cache with sudo")?;
            if !status.success() {
                anyhow::bail!("backup differs from original");
            }
            info!("Cache backup and source are the same!")
        }
        Err(err) => {
            return Err(anyhow::anyhow!("Failed to compare apt lists: {}", err));
        }
    }

    Ok(())
}

fn restore_apk_cache(cache_backup: &Path) -> Result<()> {
    if !cache_backup.exists() {
        info!("No apk cache backup found at {:?}", cache_backup);
        return Ok(());
    }

    match fs_extra::dir::copy(
        cache_backup,
        APK_CACHE_DIR,
        &fs_extra::dir::CopyOptions::new(),
    ) {
        Ok(_) => {}
        Err(err) if matches!(err.kind, fs_extra::error::ErrorKind::PermissionDenied) => {
            info!("Restoring apk cache with sudo");
            std::process::Command::new("sudo")
                .arg("cp")
                .arg("-r")
                .arg(cache_backup)
                .arg(APK_CACHE_DIR)
                .status()
                .context("Failed to restore apk cache with sudo")?;
        }
        Err(err) => {
            return Err(anyhow::anyhow!("Failed to restore apk cache: {}", err));
        }
    }

    match filesystem::is_dissimilar_dirs(cache_backup, APK_CACHE_DIR) {
        Ok(is_different) => {
            if is_different {
                anyhow::bail!("backup differs from original");
            }
            info!("Cache backup and source are the same!")
        }
        Err(err) if matches!(err.kind(), Some(std::io::ErrorKind::PermissionDenied)) => {
            info!("Comparing apk cache with sudo");
            let status = std::process::Command::new("sudo")
                .arg("diff")
                .arg("-r")
                .arg(APK_CACHE_DIR)
                .arg(cache_backup)
                .status()
                .context("Failed to compare apk cache with sudo")?;
            if !status.success() {
                anyhow::bail!("backup differs from original");
            }
            info!("Cache backup and source are the same!")
        }
        Err(err) => {
            return Err(anyhow::anyhow!("Failed to compare apk cache: {}", err));
        }
    }

    Ok(())
}

fn apk_clean() -> std::process::Command {
    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("apk").arg("cache").arg("clean");
    cmd
}

fn apk_update() -> std::process::Command {
    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("apk").arg("update");
    cmd
}

fn apk_add_pkgs(packages: &[String]) -> std::process::Command {
    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("apk").arg("add").arg("--no-cache");
    for pkg in packages {
        cmd.arg(pkg);
    }
    cmd
}

fn apk_rm_cache() -> std::process::Command {
    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("rm").arg("-rf").arg(APK_CACHE_DIR);
    cmd
}

/// Install packages using apk
pub fn install(packages: &[String]) -> Result<()> {
    anyhow::ensure!(
        os::is_alpine(),
        "apk should be used on Alpine Linux distribution"
    );

    let temp_dir = TempDir::with_prefix(config::PICO_CONFIG.temp_dir_prefix)
        .context("Failed to create temp directory")?;
    let cache_backup = temp_dir.path().join("apk");

    info!("Backing up existing apk cache");
    apk_backup_cache(&cache_backup).context("Failed to backup apk cache")?;

    info!("Updating apk repositories");
    apk_update()
        .status()
        .context("Failed to update apk repositories")?;

    info!("Installing apk packages: {:?}", packages);
    apk_add_pkgs(packages)
        .status()
        .context("Failed to install apk packages")?;

    info!("Cleaning up apk cache");
    apk_clean().status().context("Failed to clean apk cache")?;

    info!("Removing apk cache directory");
    apk_rm_cache()
        .status()
        .context("Failed to remove apk cache")?;

    restore_apk_cache(&cache_backup).context("Failed to restore apk cache")?;

    if temp_dir.path().exists() {
        // Out of scope directory may remain if directory is owned by root
        // due to file permissions
        std::process::Command::new("sudo")
            .arg("rm")
            .arg("-rf")
            .arg(temp_dir.path())
            .status()
            .context("Failed to remove temporary directory")?;
    }

    Ok(())
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
