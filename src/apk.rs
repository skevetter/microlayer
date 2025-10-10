use crate::utils::{command, linux_info};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

const APK_CACHE_DIR: &str = "/var/cache/apk";

/// Install packages using apk
pub fn install(packages: &[String]) -> Result<()> {
    anyhow::ensure!(
        linux_info::is_alpine(),
        "apk should be used on Alpine Linux distribution"
    );

    let temp_dir = tempfile::tempdir().context("Failed to create temp directory")?;
    let cache_backup = temp_dir.path().join("apk");

    if Path::new(APK_CACHE_DIR).exists() {
        fs_extra::copy_items(
            &[APK_CACHE_DIR],
            &cache_backup,
            &fs_extra::dir::CopyOptions::new(),
        )
        .context("Failed to copy apk cache")?;
    }

    install_with_cleanup(packages, &cache_backup)
}

fn install_with_cleanup(packages: &[String], cache_backup: &Path) -> Result<()> {
    let pkgs: Vec<&str> = packages.iter().map(|s| s.as_str()).collect();

    command::CommandExecutor::new()
        .command("apk")
        .arg("update")
        .execute_privileged()
        .context("Failed to update apk repositories")?;
    command::CommandExecutor::new()
        .command("apk")
        .arg("add")
        .arg("--no-cache")
        .args(&pkgs)
        .execute_privileged()
        .context("Failed to install apk packages")?;
    command::CommandExecutor::new()
        .command("rm")
        .arg("-rf")
        .arg(APK_CACHE_DIR)
        .execute_privileged()
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
