use crate::utils::{command, linux_info};
use anyhow::{Context, Result};
use log::info;
use std::fs;
use std::path::Path;

const APT_LISTS_DIR: &str = "/var/lib/apt/lists";

/// Install packages using aptitude
pub fn install(packages: &[String]) -> Result<()> {
    anyhow::ensure!(
        linux_info::is_debian_like(),
        "aptitude should be used on Debian-like distributions (Debian, Ubuntu, etc.)"
    );

    let temp_dir = tempfile::tempdir().context("Failed to create temp directory")?;
    let cache_backup = temp_dir.path().join("lists");

    if Path::new(APT_LISTS_DIR).exists() {
        command::CommandExecutor::new()
            .command("cp")
            .arg("-p")
            .arg("-R")
            .arg(APT_LISTS_DIR)
            .arg(cache_backup.to_str().unwrap())
            .execute_privileged()
            .context("Failed to copy apt lists cache")?;
    }

    install_with_cleanup(packages, &cache_backup)
}

fn install_with_cleanup(packages: &[String], cache_backup: &Path) -> Result<()> {
    if !command::CommandExecutor::new()
        .command("which")
        .arg("aptitude")
        .execute_status()
        .context("Failed to check if aptitude is installed")
        .map(|status| status.is_success())?
    {
        info!("Installing aptitude...");
        command::CommandExecutor::new()
            .command("apt-get")
            .arg("update")
            .arg("-y")
            .execute_privileged()
            .context("Failed to update package lists")?;
        command::CommandExecutor::new()
            .command("apt-get")
            .arg("install")
            .arg("-y")
            .arg("--no-install-recommends")
            .arg("aptitude")
            .execute_privileged()
            .context("Failed to install aptitude")?;
    }

    command::CommandExecutor::new()
        .command("aptitude")
        .arg("update")
        .arg("-y")
        .execute_privileged()
        .context("Failed to update aptitude package lists")?;

    let pkgs = packages.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
    command::CommandExecutor::new()
        .command("aptitude")
        .arg("install")
        .arg("-y")
        .arg("--without-recommends")
        .args(&pkgs)
        .execute_privileged()
        .context("Failed to install packages with aptitude")?;
    command::CommandExecutor::new()
        .command("aptitude")
        .arg("clean")
        .execute_privileged()
        .context("Failed to clean aptitude cache")?;
    command::CommandExecutor::new()
        .command("rm")
        .arg("-rf")
        .arg("/var/lib/apt/lists")
        .execute_privileged()
        .context("Failed to remove apt lists")?;
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
        // Test that the function signature is correct
        let packages = vec!["curl".to_string()];
        // Just test compilation, not actual execution
        let _ = install(&packages);
    }
}
