use crate::utils::{command, linux_info};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Install packages using apk with minimal layer footprint
pub fn install(packages: &[String]) -> Result<()> {
    anyhow::ensure!(
        linux_info::is_alpine(),
        "apk should be used on Alpine Linux distribution"
    );

    // Create temp directory for cache backup
    let temp_dir = tempfile::tempdir().context("Failed to create temp directory")?;
    let cache_backup = temp_dir.path().join("apk");

    // Backup apk cache
    if Path::new("/var/cache/apk").exists() {
        command::execute(&format!(
            "cp -p -R /var/cache/apk {}",
            cache_backup.display()
        ))?;
    }

    // Install packages with cleanup
    install_with_cleanup(packages, &cache_backup)
}

fn install_with_cleanup(packages: &[String], cache_backup: &Path) -> Result<()> {
    // Update package index
    command::execute("apk update")?;

    // Install packages
    let pkg_list = packages.join(" ");
    command::execute(&format!("apk add --no-cache {}", pkg_list))?;

    // Restore cache
    command::execute("rm -rf /var/cache/apk")?;
    if cache_backup.exists() {
        fs::rename(cache_backup, "/var/cache/apk").context("Failed to restore apk cache")?;
    }

    Ok(())
}
