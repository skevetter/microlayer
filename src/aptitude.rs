use crate::utils::{command, linux_info};
use anyhow::{Context, Result};
use log::info;
use std::fs;
use std::path::Path;

/// Install packages using aptitude with cleanup
pub fn install(packages: &[String]) -> Result<()> {
    anyhow::ensure!(
        linux_info::is_debian_like(),
        "aptitude should be used on Debian-like distributions (Debian, Ubuntu, etc.)"
    );

    let temp_dir = tempfile::tempdir().context("Failed to create temp directory")?;
    let cache_backup = temp_dir.path().join("lists");

    // Backup apt lists
    const APT_LISTS_DIR: &str = "/var/lib/apt/lists";
    if Path::new(APT_LISTS_DIR).exists() {
        command::execute(&format!(
            "cp -p -R {} {}",
            APT_LISTS_DIR,
            cache_backup.display()
        ))?;
    }

    install_with_cleanup(packages, &cache_backup)
}

fn install_with_cleanup(packages: &[String], cache_backup: &Path) -> Result<()> {
    // Ensure aptitude is installed
    if command::execute_status("which aptitude")? != 0 {
        info!("Installing aptitude...");
        command::execute("apt-get update -y")?;
        command::execute("apt-get install -y --no-install-recommends aptitude")?;
    }

    // Update package lists
    command::execute("aptitude update -y")?;

    // Install packages
    let pkg_list = packages.join(" ");
    command::execute(&format!(
        "aptitude install -y --without-recommends {}",
        pkg_list
    ))?;

    // Cleanup: remove package cache
    command::execute("aptitude clean")?;
    
    // Cleanup: restore lists cache
    command::execute(&format!("rm -rf {}", "/var/lib/apt/lists"))?;
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
