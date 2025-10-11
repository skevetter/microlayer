use crate::apt_get;
use crate::utils::linux_info;
use anyhow::{Context, Result};
use log::{info, warn};
use std::fs;
use std::path::Path;

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
    if Path::new(APT_LISTS_DIR).exists() {
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

    let temp_dir = tempfile::tempdir().context("Failed to create temp directory")?;
    let cache_backup = temp_dir.path().join("lists");

    apt_backup_lists(&cache_backup)?;
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

    apt_install_packages(packages)
        .status()
        .context("Failed to install apt packages")?;
    remove_ppas(&installed_ppas)
        .status()
        .context("Failed to remove PPAs")?;
    apt_purge(installed_ppa_packages.as_slice())
        .status()
        .context("Failed to purge package installed for PPA support")?;
    apt_clean().status().context("Failed to clean apt cache")?;
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

    #[test]
    fn test_install_function_signature() {
        let packages = vec!["curl".to_string()];
        let _ = install(&packages, None, false);
    }
}
