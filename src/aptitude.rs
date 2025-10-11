use crate::apt_get;
use crate::utils::linux_info;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

struct Aptitude {}

impl Aptitude {
    const APT_LISTS_DIR: &str = "/var/lib/apt/lists";

    fn get_aptitude() -> Result<()> {
        apt_get::install(&["aptitude".to_string()], Some(&[]), false)
            .context("Failed to install aptitude")?;
        Ok(())
    }

    fn update() -> std::process::Command {
        let mut cmd = std::process::Command::new("sudo");
        cmd.arg("aptitude").arg("update");
        cmd
    }

    fn clean() -> std::process::Command {
        let mut cmd = std::process::Command::new("sudo");
        cmd.arg("aptitude").arg("clean");
        cmd
    }

    fn backup_apt_lists(cache_backup: &Path) -> Result<()> {
        if Path::new(Aptitude::APT_LISTS_DIR).exists() {
            let mut cmd = std::process::Command::new("sudo");
            cmd.arg("cp")
                .arg("-p")
                .arg("-R")
                .arg(Aptitude::APT_LISTS_DIR)
                .arg(cache_backup.to_str().unwrap())
                .status()
                .context("Failed to copy apt lists cache")?;
        }
        Ok(())
    }

    fn rm_apt_cache() -> std::process::Command {
        let mut cmd = std::process::Command::new("sudo");
        cmd.arg("rm").arg("-rf").arg(Aptitude::APT_LISTS_DIR);
        cmd
    }

    fn install(packages: &[String]) -> std::process::Command {
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
}

/// Install packages using aptitude
pub fn install(packages: &[String]) -> Result<()> {
    anyhow::ensure!(
        linux_info::is_debian_like(),
        "aptitude should be used on Debian-like distributions (Debian, Ubuntu, etc.)"
    );

    let temp_dir = tempfile::tempdir().context("Failed to create temp directory")?;
    let cache_backup = temp_dir.path().join("lists");

    Aptitude::backup_apt_lists(&cache_backup)?;
    Aptitude::update()
        .status()
        .context("Failed to update aptitude repositories")?;
    Aptitude::get_aptitude().context("Failed to install aptitude")?;
    Aptitude::install(packages)
        .status()
        .context("Failed to install packages with aptitude")?;
    Aptitude::clean()
        .status()
        .context("Failed to clean aptitude cache")?;
    Aptitude::rm_apt_cache()
        .status()
        .context("Failed to remove aptitude cache")?;

    if cache_backup.exists() {
        fs::rename(cache_backup, "/var/lib/apt/lists")
            .context("Failed to restore apt lists cache")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_function_signature() {
        let packages = vec!["curl".to_string()];
        let _ = install(&packages);
    }
}
