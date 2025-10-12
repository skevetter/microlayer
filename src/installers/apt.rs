use crate::installers::apt_get;
use crate::utils::{filesystem, os};
use anyhow::{Context, Result};
use log::{info, warn};
use std::path::Path;
use tempfile::TempDir;

const APT_LISTS_DIR: &str = "/var/lib/apt/lists";

fn apt_update() -> std::process::Command {
    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("apt").arg("update").arg("-y");
    cmd
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
    if !Path::new(APT_LISTS_DIR).exists() {
        info!("No existing apt lists cache to back up");
        return Ok(());
    }

    info!("Backing up apt lists cache to {:?}", cache_backup);
    match fs_extra::copy_items(
        &[APT_LISTS_DIR],
        cache_backup,
        &fs_extra::dir::CopyOptions::new(),
    ) {
        Ok(_) => {}
        Err(err) if matches!(err.kind, fs_extra::error::ErrorKind::PermissionDenied) => {
            info!("Backing up apt lists with sudo");
            std::process::Command::new("sudo")
                .arg("cp")
                .arg("-r")
                .arg(APT_LISTS_DIR)
                .arg(cache_backup.to_str().unwrap())
                .status()
                .context("Failed to copy apt lists cache")?;
        }
        Err(err) => {
            return Err(anyhow::anyhow!("Failed to backup apt lists: {}", err));
        }
    }

    match filesystem::is_dissimilar_dirs(cache_backup, APT_LISTS_DIR) {
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
                .arg(APT_LISTS_DIR)
                .arg(cache_backup)
                .status()
                .context("Failed to compare apt lists with sudo")?;
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

fn restore_apt_lists(cache_backup: &Path) -> Result<()> {
    if !cache_backup.exists() {
        info!("No apt lists backup found at {:?}", cache_backup);
        return Ok(());
    }

    match fs_extra::dir::copy(
        cache_backup,
        APT_LISTS_DIR,
        &fs_extra::dir::CopyOptions::new(),
    ) {
        Ok(_) => {}
        Err(err) if matches!(err.kind, fs_extra::error::ErrorKind::PermissionDenied) => {
            info!("Restoring apt lists with sudo");
            std::process::Command::new("sudo")
                .arg("cp")
                .arg("-r")
                .arg(cache_backup)
                .arg(APT_LISTS_DIR)
                .status()
                .context("Failed to restore apt lists with sudo")?;
        }
        Err(err) => {
            return Err(anyhow::anyhow!("Failed to restore apt lists: {}", err));
        }
    }

    match filesystem::is_dissimilar_dirs(cache_backup, APT_LISTS_DIR) {
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
                .arg(APT_LISTS_DIR)
                .arg(cache_backup)
                .status()
                .context("Failed to compare apt lists with sudo")?;
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
        os::is_debian_like(),
        "apt should be used on Debian-like distributions (Debian, Ubuntu, etc.)"
    );

    let mut ppas = ppas.map(|p| p.to_vec()).unwrap_or_default();

    if !ppas.is_empty() && !os::is_ubuntu() && !force_ppas_on_non_ubuntu {
        warn!("PPAs are ignored on non-Ubuntu distros!");
        info!("Use --force-ppas-on-non-ubuntu to include them anyway.");
        ppas.clear();
    }

    let temp_dir = TempDir::with_prefix("picolayer_").context("Failed to create temp directory")?;
    let cache_backup = temp_dir.path().join("apt");
    info!("Backup path: {:?}", cache_backup);

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

    restore_apt_lists(&cache_backup).context("Failed to restore apt lists")?;

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
    fn test_apt() {
        let packages = vec!["curl".to_string()];
        let _ = install(&packages, None, false);
    }
}
