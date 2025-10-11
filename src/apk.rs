use crate::utils::linux_info;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

const APK_CACHE_DIR: &str = "/var/cache/apk";

fn apk_backup_cache(cache_backup: &Path) -> Result<()> {
    if Path::new(APK_CACHE_DIR).exists() {
        fs_extra::copy_items(
            &[APK_CACHE_DIR],
            cache_backup,
            &fs_extra::dir::CopyOptions::new(),
        )
        .context("Failed to copy apk cache")?;
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
        linux_info::is_alpine(),
        "apk should be used on Alpine Linux distribution"
    );

    let temp_dir = tempfile::tempdir().context("Failed to create temp directory")?;
    let cache_backup = temp_dir.path().join("apk");

    apk_backup_cache(&cache_backup).context("Failed to backup apk cache")?;
    apk_update()
        .status()
        .context("Failed to update apk repositories")?;
    apk_add_pkgs(packages)
        .status()
        .context("Failed to install apk packages")?;
    apk_clean().status().context("Failed to clean apk cache")?;
    apk_rm_cache()
        .status()
        .context("Failed to remove apk cache")?;

    if cache_backup.exists() {
        fs::rename(cache_backup, APK_CACHE_DIR).context("Failed to restore apk cache")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_function_signature() {
        let packages = vec!["test".to_string()];
        let result = install(&packages);
        let _ = result;
    }
}
