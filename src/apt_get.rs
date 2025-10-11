use crate::utils::linux_info;
use anyhow::{Context, Result};
use log::{info, warn};
use std::path::Path;

const PPA_SUPPORT_PACKAGES: &[&str] = &["software-properties-common"];
const PPA_SUPPORT_PACKAGES_DEBIAN: &[&str] = &["python3-launchpadlib"];
const APT_LISTS_DIR: &str = "/var/lib/apt/lists";

fn apt_install(packages: &[String]) -> std::process::Command {
    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("apt-get")
        .arg("install")
        .arg("-y")
        .arg("--no-install-recommends")
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .stdin(std::process::Stdio::inherit());
    for pkg in packages {
        cmd.arg(pkg);
    }
    cmd
}

fn apt_update() -> std::process::Command {
    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("apt-get").arg("update").arg("-y");
    cmd
}

fn apt_get() -> std::process::Command {
    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("apt-get");
    cmd
}

fn apt_add_repository() -> std::process::Command {
    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("add-apt-repository");
    cmd
}

fn apt_remove_ppas(ppas: &[String]) -> std::process::Command {
    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("add-apt-repository").arg("-y").arg("--remove");
    for ppa in ppas {
        cmd.arg(ppa);
    }
    cmd
}

fn apt_rm_cache() -> std::process::Command {
    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("rm").arg("-rf").arg(APT_LISTS_DIR);
    cmd
}

fn apt_purge(packages: &[String]) -> std::process::Command {
    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("apt-get")
        .arg("purge")
        .arg("-y")
        .arg("--auto-remove");
    for pkg in packages {
        cmd.arg(pkg);
    }
    cmd
}

fn dpkg() -> std::process::Command {
    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("dpkg");
    cmd
}

fn backup_apt_lists(cache_backup: &Path) -> Result<()> {
    if Path::new(APT_LISTS_DIR).exists() {
        let options = fs_extra::dir::CopyOptions::new()
            .overwrite(true)
            .copy_inside(true);
        match fs_extra::dir::copy(APT_LISTS_DIR, cache_backup, &options) {
            Ok(_) => {}
            Err(err) if matches!(err.kind, fs_extra::error::ErrorKind::PermissionDenied) => {
                std::process::Command::new("sudo")
                    .arg("cp")
                    .arg("-r")
                    .arg(APT_LISTS_DIR)
                    .arg(cache_backup)
                    .status()
                    .context("Failed to backup apt lists with sudo")?;
            }
            Err(err) => {
                return Err(anyhow::anyhow!("Failed to backup apt lists: {}", err));
            }
        }
    }
    Ok(())
}

fn restore_apt_lists(cache_backup: &Path) -> Result<()> {
    if cache_backup.exists() {
        let options = fs_extra::dir::CopyOptions::new()
            .overwrite(true)
            .copy_inside(true);
        match fs_extra::dir::copy(cache_backup, APT_LISTS_DIR, &options) {
            Ok(_) => {}
            Err(err) if matches!(err.kind, fs_extra::error::ErrorKind::PermissionDenied) => {
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
    }
    Ok(())
}

/// Install packages using apt-get with optional PPAs
pub fn install(
    packages: &[String],
    ppas: Option<&[String]>,
    force_ppas_on_non_ubuntu: bool,
) -> Result<()> {
    anyhow::ensure!(
        linux_info::is_debian_like(),
        "apt-get should be used on Debian-like distributions (Debian, Ubuntu, etc.)"
    );

    let mut ppas = ppas.map(|p| p.to_vec()).unwrap_or_default();

    if !ppas.is_empty() && !linux_info::is_ubuntu() && !force_ppas_on_non_ubuntu {
        warn!("PPAs are ignored on non-Ubuntu distros!");
        info!("Use --force-ppas-on-non-ubuntu to include them anyway.");
        ppas.clear();
    }

    let temp_dir = tempfile::tempdir().context("Failed to create temp directory")?;
    let cache_backup = temp_dir.path().join("lists");

    backup_apt_lists(&cache_backup)?;
    apt_update()
        .status()
        .context("Failed to update repositories")?;

    let mut installed_ppas = Vec::new();
    let mut installed_ppa_packages = Vec::new();

    if !ppas.is_empty() {
        let (ppas_added, ppa_pkgs) = add_ppas(&ppas)?;
        installed_ppas = ppas_added;
        installed_ppa_packages = ppa_pkgs;
    }

    apt_install(packages)
        .status()
        .context("Failed to install apt packages")?;
    if !ppas.is_empty() {
        apt_remove_ppas(&installed_ppas)
            .status()
            .context("Failed to remove PPAs")?;
        apt_purge(installed_ppa_packages.as_slice())
            .status()
            .context("Failed to purge packages installed for PPA support")?;
    }
    apt_get()
        .arg("clean")
        .status()
        .context("Failed to clean apt cache")?;
    apt_rm_cache()
        .arg(APT_LISTS_DIR)
        .status()
        .context("Failed to remove apt lists cache")?;

    if cache_backup.exists() {
        restore_apt_lists(&cache_backup).context("Failed to restore apt lists")?;
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

    let required_packages: Vec<&str> = if linux_info::is_ubuntu() {
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

        if !status
            && std::process::Command::new("which")
                .arg(pkg)
                .status()
                .map(|o| o.success())?
        {
            apt_install(&[pkg.to_string()])
                .status()
                .context("Failed to install packages required for PPA support")?;
            installed_packages.push(pkg.to_string());
        }
    }

    if std::process::Command::new("which")
        .arg("add-apt-repository")
        .status()
        .map(|o| o.success())?
    {
        for ppa in &normalized_ppas {
            apt_add_repository()
                .arg("-y")
                .arg(ppa)
                .status()
                .context("Failed to add PPA")?;
            installed_ppas.push(ppa.clone());
        }
    } else {
        warn!("add-apt-repository command not found");
    }

    apt_get()
        .arg("update")
        .arg("-y")
        .status()
        .context("Failed to update apt repositories after adding PPAs")?;

    Ok((installed_ppas, installed_packages))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_install_function_signature() {
        let packages = vec!["test".to_string()];
        let result = install(&packages, None, false);
        let _ = result;
    }
}
