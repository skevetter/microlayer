use crate::utils::linux_info;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

struct Apk {}

impl Apk {
    const APK_CACHE_DIR: &str = "/var/cache/apk";

    fn backup_apk_cache(cache_backup: &Path) -> Result<()> {
        if Path::new(Apk::APK_CACHE_DIR).exists() {
            fs_extra::copy_items(
                &[Apk::APK_CACHE_DIR],
                cache_backup,
                &fs_extra::dir::CopyOptions::new(),
            )
            .context("Failed to copy apk cache")?;
        }
        Ok(())
    }

    fn clean() -> std::process::Command {
        let mut cmd = std::process::Command::new("sudo");
        cmd.arg("apk").arg("cache").arg("clean");
        cmd
    }

    fn update() -> std::process::Command {
        let mut cmd = std::process::Command::new("sudo");
        cmd.arg("apk").arg("update");
        cmd
    }

    fn add_pkgs(packages: &[String]) -> std::process::Command {
        let mut cmd = std::process::Command::new("sudo");
        cmd.arg("apk").arg("add").arg("--no-cache");
        for pkg in packages {
            cmd.arg(pkg);
        }
        cmd
    }

    fn rm_apk_cache() -> std::process::Command {
        let mut cmd = std::process::Command::new("sudo");
        cmd.arg("rm").arg("-rf").arg(Self::APK_CACHE_DIR);
        cmd
    }
}

/// Install packages using apk
pub fn install(packages: &[String]) -> Result<()> {
    anyhow::ensure!(
        linux_info::is_alpine(),
        "apk should be used on Alpine Linux distribution"
    );

    let temp_dir = tempfile::tempdir().context("Failed to create temp directory")?;
    let cache_backup = temp_dir.path().join("apk");

    Apk::backup_apk_cache(&cache_backup).context("Failed to backup apk cache")?;
    Apk::update()
        .status()
        .context("Failed to update apk repositories")?;
    Apk::add_pkgs(packages)
        .status()
        .context("Failed to install apk packages")?;
    Apk::clean().status().context("Failed to clean apk cache")?;
    Apk::rm_apk_cache()
        .status()
        .context("Failed to remove apk cache")?;

    if cache_backup.exists() {
        fs::rename(cache_backup, Apk::APK_CACHE_DIR).context("Failed to restore apk cache")?;
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
