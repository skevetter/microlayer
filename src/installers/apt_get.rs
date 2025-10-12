use crate::config;
use crate::utils::{filesystem, os};
use anyhow::{Context, Error, Result};
use log::{info, warn};
use std::path::Path;
use tempfile::TempDir;

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

fn apt_rm_cache() -> Result<(), Error> {
    match fs_extra::remove_items(&[APT_LISTS_DIR]) {
        Ok(_) => Ok(()),
        Err(err) if matches!(err.kind, fs_extra::error::ErrorKind::PermissionDenied) => {
            info!("Removing apt lists with sudo");
            std::process::Command::new("sudo")
                .arg("rm")
                .arg("-rf")
                .arg(APT_LISTS_DIR)
                .status()
                .context("Failed to remove apt lists with sudo")?;
            Ok(())
        }
        Err(err) => Err(anyhow::anyhow!("Failed to remove apt lists: {}", err)),
    }
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
    if !Path::new(APT_LISTS_DIR).exists() {
        info!("No existing apt lists to back up");
        return Ok(());
    }

    // let items: Vec<String> = std::fs::read_dir(APT_LISTS_DIR)
    //     .context("Failed to read apt lists directory")?
    //     .filter_map(|e| e.ok())
    //     .map(|e| e.path().to_string_lossy().into_owned())
    //     .collect();
    let dir_content = fs_extra::dir::get_dir_content(APT_LISTS_DIR)
        .context("Failed to list apt lists directory")?;

    // fs_extra::copy_items expects a slice of items that implement AsRef<Path>.
    // dir_content.files is a Vec<String>, so convert to Vec<PathBuf>.
    let mut items: Vec<std::path::PathBuf> = dir_content
        .files
        .iter()
        .map(std::path::PathBuf::from)
        .collect();

    dir_content
        .directories
        .iter()
        .for_each(|d| items.push(std::path::PathBuf::from(d)));

    match fs_extra::copy_items(
        &items,
        cache_backup,
        &fs_extra::dir::CopyOptions::new().overwrite(true),
    ) {
        Ok(_) => {}
        Err(err) if matches!(err.kind, fs_extra::error::ErrorKind::PermissionDenied) => {
            info!("Backing up apt lists with sudo");
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

    match filesystem::is_dissimilar_dirs(cache_backup.join("lists"), APT_LISTS_DIR) {
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

    // Create the lists directory to restore into
    if !Path::new(APT_LISTS_DIR).exists() {
        std::fs::create_dir_all(APT_LISTS_DIR).context("Failed to recreate apt lists directory")?;
    }

    // Assert the APT_LISTS_DIR is empty before restoring
    assert!(
        std::fs::read_dir(APT_LISTS_DIR)
            .context("Failed to read apt lists directory")?
            .next()
            .is_none(),
        "APT lists directory is not empty before restore"
    );

    let dir_content = fs_extra::dir::get_dir_content(cache_backup)
        .context("Failed to list apt lists backup directory")?;

    let mut files = dir_content
        .files
        .iter()
        .map(std::path::PathBuf::from)
        .collect::<Vec<std::path::PathBuf>>();

    dir_content
        .directories
        .iter()
        .for_each(|d| files.push(std::path::PathBuf::from(d)));

    match fs_extra::copy_items(
        &files,
        APT_LISTS_DIR,
        &fs_extra::dir::CopyOptions::new().overwrite(true),
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

    match filesystem::is_dissimilar_dirs(cache_backup.join("lists"), APT_LISTS_DIR) {
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

/// Install packages using apt-get with optional PPAs
pub fn install(
    packages: &[String],
    ppas: Option<&[String]>,
    force_ppas_on_non_ubuntu: bool,
) -> Result<()> {
    anyhow::ensure!(
        os::is_debian_like(),
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
    info!("Backup path {:?}", cache_backup);
    std::fs::create_dir_all(&cache_backup).context("Failed to create backup directory")?;

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

    info!("Installing apt packages: {:?}", packages);
    apt_install(packages)
        .status()
        .context("Failed to install apt packages")?;

    if !ppas.is_empty() {
        info!("Removing added PPAs and packages installed for PPA support");
        apt_remove_ppas(&installed_ppas)
            .status()
            .context("Failed to remove PPAs")?;
        apt_purge(installed_ppa_packages.as_slice())
            .status()
            .context("Failed to purge packages installed for PPA support")?;
    }

    info!("Cleaning up apt cache");
    apt_get()
        .arg("clean")
        .status()
        .context("Failed to clean apt cache")?;

    info!("Cleaning up apt lists cache");
    apt_rm_cache().context("Failed to remove apt lists cache")?;

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
    fn test_apt_get() {
        let packages = vec!["test".to_string()];
        let result = install(&packages, None, false);
        let _ = result;
    }
}
