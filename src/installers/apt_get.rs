use crate::config;
use crate::utils::os;
use anyhow::{Context, Result};
use log::{debug, info, warn};
use std::path::Path;
use tempfile::TempDir;

const PPA_SUPPORT_PACKAGES: &[&str] = &["software-properties-common"];
const PPA_SUPPORT_PACKAGES_DEBIAN: &[&str] = &["python3-launchpadlib"];
const APT_LISTS_DIR: &str = "/var/lib/apt/lists";

/// Install packages using apt-get with optional PPAs
pub fn install(
    packages: &[String],
    ppas: Option<&[String]>,
    force_ppas_on_non_ubuntu: bool,
) -> Result<()> {
    anyhow::ensure!(
        os::is_debian_like() && which::which("apt-get").is_ok(),
        "apt-get should be used on Debian-like distributions (Debian, Ubuntu, etc.)"
    );

    let mut ppas = ppas.map(|p| p.to_vec()).unwrap_or_default();
    if !ppas.is_empty() && !os::is_ubuntu() && !force_ppas_on_non_ubuntu {
        warn!("PPAs are ignored on non-Ubuntu distros!");
        info!("Use --force-ppas-on-non-ubuntu to include them anyway.");
        ppas.clear();
    }

    let temp_dir = TempDir::with_prefix(config::PICO_CONFIG.temp_dir_prefix)
        .context("Failed to create temp directory")?;
    let cache_backup = temp_dir.path().join("apt_get");
    debug!("Backup path {:?}", cache_backup);

    match os::copy_files(Path::new(APT_LISTS_DIR), &cache_backup) {
        Ok(_) => {}
        Err(e) => {
            anyhow::bail!("Failed to back up apt lists: {}", e);
        }
    }

    debug!("Updating apt repositories");
    apt_update()
        .output()
        .map(|o| debug!("Apt update output: {:?}", o))
        .context("Failed to update repositories")?;

    let mut installed_ppas = Vec::new();
    let mut installed_ppa_packages = Vec::new();

    if !ppas.is_empty() {
        let (ppas_added, ppa_pkgs) = add_ppas(&ppas)?;
        installed_ppas = ppas_added;
        installed_ppa_packages = ppa_pkgs;
    }

    debug!("Installing apt packages: {:?}", packages);
    apt_install(packages)
        .output()
        .map(|o| debug!("Apt install output: {:?}", o))
        .context("Failed to install apt packages")?;

    if !ppas.is_empty() {
        debug!("Removing added PPAs and packages installed for PPA support");
        apt_remove_ppas(&installed_ppas)
            .output()
            .map(|o| debug!("Apt remove PPAs output: {:?}", o))
            .context("Failed to remove PPAs")?;
        apt_purge(installed_ppa_packages.as_slice())
            .output()
            .map(|o| debug!("Apt purge output: {:?}", o))
            .context("Failed to purge packages installed for PPA support")?;
    }

    debug!("Cleaning up apt cache");
    apt_get()
        .arg("clean")
        .output()
        .map(|o| debug!("Apt clean output: {:?}", o))
        .context("Failed to clean apt cache")?;

    match os::copy_files(&cache_backup, Path::new(APT_LISTS_DIR)) {
        Ok(_) => {}
        Err(e) => {
            anyhow::bail!("Failed to restore apt lists from backup: {}", e);
        }
    }

    if temp_dir.path().exists() {
        anyhow::ensure!(
            temp_dir.close().is_ok(),
            "Temporary directory could not be deleted"
        );
    } else {
        debug!("Temporary directory is deleted");
    }

    Ok(())
}

pub fn add_ppas(ppas: &[String]) -> Result<(Vec<String>, Vec<String>)> {
    let mut installed_ppas = Vec::new();
    let mut installed_packages = Vec::new();

    let normalized_ppas: Vec<String> = ppas
        .iter()
        .map(|ppa| {
            if ppa.starts_with("ppa:") {
                ppa.clone()
            } else {
                format!("ppa:{}", ppa)
            }
        })
        .collect();

    let required_packages: Vec<&str> = if os::is_ubuntu() {
        PPA_SUPPORT_PACKAGES.to_vec()
    } else {
        PPA_SUPPORT_PACKAGES
            .iter()
            .chain(PPA_SUPPORT_PACKAGES_DEBIAN.iter())
            .copied()
            .collect()
    };

    for pkg in required_packages {
        let status = dpkg().arg("-s").arg(pkg).status().map(|o| o.success())?;

        if !status && which::which(pkg).is_err() {
            apt_install(&[pkg.to_string()])
                .status()
                .context("Failed to install packages required for PPA support")?;
            installed_packages.push(pkg.to_string());
        }
    }

    if which::which("add-apt-repository").is_ok() {
        for ppa in &normalized_ppas {
            apt_add_repository()
                .arg("-y")
                .arg(ppa)
                .output()
                .map(|o| debug!("Apt add PPA output: {:?}", o))
                .context("Failed to add PPA")?;
            installed_ppas.push(ppa.clone());
        }
    } else {
        warn!("add-apt-repository command not found");
    }

    apt_update()
        .output()
        .map(|o| debug!("Apt update output: {:?}", o))
        .context("Failed to update repositories")?;

    Ok((installed_ppas, installed_packages))
}

fn apt_get() -> std::process::Command {
    std::process::Command::new("apt-get")
}

fn apt_install(packages: &[String]) -> std::process::Command {
    let mut cmd = apt_get();
    cmd.arg("install")
        .arg("-y")
        .arg("--no-install-recommends")
        .args(packages);
    cmd
}

fn apt_update() -> std::process::Command {
    let mut cmd = apt_get();
    cmd.arg("update").arg("-y");
    cmd
}

fn apt_add_repository() -> std::process::Command {
    std::process::Command::new("add-apt-repository")
}

fn apt_remove_ppas(ppas: &[String]) -> std::process::Command {
    let mut cmd = apt_add_repository();
    cmd.arg("-y").arg("--remove").args(ppas);
    cmd
}

fn apt_purge(packages: &[String]) -> std::process::Command {
    let mut cmd = apt_get();
    cmd.arg("purge")
        .arg("-y")
        .arg("--auto-remove")
        .args(packages);
    cmd
}

fn dpkg() -> std::process::Command {
    std::process::Command::new("dpkg")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_apt_get() {
        env_logger::builder().is_test(true).try_init().ok();
        let packages = vec!["curl".to_string()];
        let result = install(&packages, None, false);
        let _ = result;
    }

    #[test]
    #[serial]
    fn test_apt_get_invalid_package() {
        env_logger::builder().is_test(true).try_init().ok();
        let packages = vec!["test".to_string()];
        let result = install(&packages, None, false);
        let _ = result;
    }
}
