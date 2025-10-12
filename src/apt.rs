use crate::apt_get;
use crate::utils::linux_info;
use anyhow::{Context, Result};
use log::{info, warn};
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

const APT_LISTS_DIR: &str = "/var/lib/apt/lists";

fn apt_update() -> std::process::Command {
    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("apt").arg("update").arg("-y");
    cmd
}

/// Wait for apt/dpkg lock to be released
fn wait_for_apt_lock() -> Result<()> {
    const MAX_RETRIES: u32 = 30;
    const RETRY_DELAY_SECS: u64 = 2;
    
    for i in 0..MAX_RETRIES {
        // Check if lock files exist
        let apt_lock_exists = Path::new("/var/lib/apt/lists/lock").exists();
        let dpkg_lock_exists = Path::new("/var/lib/dpkg/lock").exists();
        let dpkg_frontend_lock_exists = Path::new("/var/lib/dpkg/lock-frontend").exists();
        
        if !apt_lock_exists && !dpkg_lock_exists && !dpkg_frontend_lock_exists {
            return Ok(());
        }
        
        if i == 0 {
            info!("Waiting for apt/dpkg locks to be released...");
        }
        
        thread::sleep(Duration::from_secs(RETRY_DELAY_SECS));
    }
    
    warn!("apt/dpkg locks still present after waiting, proceeding anyway");
    Ok(())
}

fn apt_install_packages(packages: &[String]) -> std::process::Command {
    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("apt")
        .arg("install")
        .arg("-y")
        .arg("--no-install-recommends");
    for pkg in packages {
        cmd.arg(pkg);
    }
    cmd
}

fn apt_backup_lists(cache_backup: &Path) -> Result<()> {
    if Path::new(APT_LISTS_DIR).exists() {
        info!("Backing up apt lists cache to {:?}", cache_backup);
        std::process::Command::new("sudo ")
            .arg("cp")
            .arg("-p")
            .arg("-R")
            .arg(APT_LISTS_DIR)
            .arg(cache_backup.to_str().unwrap())
            .status()
            .context("Failed to copy apt lists cache")?;
    }
    Ok(())
}

fn apt_purge(packages: &[String]) -> std::process::Command {
    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("apt").arg("purge").arg("-y").arg("--auto-remove");
    for pkg in packages {
        cmd.arg(pkg);
    }
    cmd
}

fn apt_rm_cache() -> std::process::Command {
    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("rm").arg("-rf").arg(APT_LISTS_DIR);
    cmd
}

fn apt_clean() -> std::process::Command {
    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("apt").arg("clean");
    cmd
}

fn remove_ppas(ppas: &[String]) -> std::process::Command {
    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("add-apt-repository").arg("-y").arg("--remove");
    for ppa in ppas {
        cmd.arg(ppa);
    }
    cmd
}

/// Install packages using apt
pub fn install(
    packages: &[String],
    ppas: Option<&[String]>,
    force_ppas_on_non_ubuntu: bool,
) -> Result<()> {
    anyhow::ensure!(
        linux_info::is_debian_like(),
        "apt should be used on Debian-like distributions (Debian, Ubuntu, etc.)"
    );

    let mut ppas = ppas.map(|p| p.to_vec()).unwrap_or_default();

    if !ppas.is_empty() && !linux_info::is_ubuntu() && !force_ppas_on_non_ubuntu {
        warn!("PPAs are ignored on non-Ubuntu distros!");
        info!("Use --force-ppas-on-non-ubuntu to include them anyway.");
        ppas.clear();
    }

    let temp_dir = TempDir::with_prefix("picolayer_").context("Failed to create temp directory")?;
    let cache_backup = temp_dir.path().join("apt");

    wait_for_apt_lock()?;
    
    info!("Backing up existing apt lists cache");
    apt_backup_lists(&cache_backup)?;

    info!("Updating apt repositories");
    apt_update()
        .status()
        .context("Failed to update apt repositories")?;

    let mut installed_ppas = Vec::new();
    let mut installed_ppa_packages = Vec::new();

    if !ppas.is_empty() {
        let (ppas_added, ppa_pkgs) = apt_get::add_ppas(&ppas)?;
        installed_ppas.extend(ppas_added);
        installed_ppa_packages.extend(ppa_pkgs);
    }

    info!("Installing apt packages: {:?}", packages);
    apt_install_packages(packages)
        .status()
        .context("Failed to install apt packages")?;

    info!("Removing added PPAs and packages installed for PPA support");
    remove_ppas(&installed_ppas)
        .status()
        .context("Failed to remove PPAs")?;

    info!("Purging packages installed for PPA support");
    apt_purge(installed_ppa_packages.as_slice())
        .status()
        .context("Failed to purge package installed for PPA support")?;

    info!("Cleaning up apt cache");
    apt_clean().status().context("Failed to clean apt cache")?;

    info!("Removing apt lists cache directory");
    apt_rm_cache()
        .status()
        .context("Failed to remove apt lists cache")?;

    if cache_backup.exists() {
        fs::rename(cache_backup, APT_LISTS_DIR).context("Failed to restore apt lists cache")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_apt() {
        let packages = vec!["curl".to_string()];
        let _ = install(&packages, None, false);
    }
}
