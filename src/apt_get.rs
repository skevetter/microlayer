use crate::utils::{command, linux_info};
use anyhow::{Context, Result};
use log::{info, warn};
use std::path::Path;

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

    install_with_cleanup(packages, &ppas, &cache_backup)
}

fn install_with_cleanup(packages: &[String], ppas: &[String], cache_backup: &Path) -> Result<()> {
    command::CommandExecutor::new()
        .command("apt-get")
        .arg("update")
        .arg("-y")
        .execute_privileged()
        .context("Failed to update apt repositories")?;

    let mut installed_ppas = Vec::new();
    let mut installed_ppa_packages = Vec::new();

    if !ppas.is_empty() {
        let (ppas_added, ppa_pkgs) = add_ppas(ppas)?;
        installed_ppas = ppas_added;
        installed_ppa_packages = ppa_pkgs;
    }

    let pkgs: Vec<&str> = packages.iter().map(|s| s.as_str()).collect();

    command::CommandExecutor::new()
        .command("apt-get")
        .arg("install")
        .arg("-y")
        .arg("--no-install-recommends")
        .args(&pkgs)
        .execute_privileged()
        .context("Failed to install apt packages")?;

    for ppa in &installed_ppas {
        command::CommandExecutor::new()
            .command("add-apt-repository")
            .arg("-y")
            .arg("--remove")
            .arg(ppa)
            .execute_privileged()
            .context("Failed to remove PPA")?;
    }

    for pkg in &installed_ppa_packages {
        command::CommandExecutor::new()
            .command("apt-get")
            .arg("-y")
            .arg("purge")
            .arg(pkg)
            .arg("--auto-remove")
            .execute_privileged()
            .context("Failed to purge package")?;
    }

    command::CommandExecutor::new()
        .command("apt-get")
        .arg("clean")
        .execute_privileged()
        .context("Failed to clean apt cache")?;
    command::CommandExecutor::new()
        .command("rm")
        .arg("-rf")
        .arg(APT_LISTS_DIR)
        .execute_privileged()
        .context("Failed to remove apt lists cache")?;
    if cache_backup.exists() {
        fs_extra::move_items(
            &[cache_backup],
            APT_LISTS_DIR,
            &fs_extra::dir::CopyOptions::new(),
        )
        .context("Failed to restore apt lists cache")?;
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
        let status = command::CommandBuilder::new("dpkg")
            .arg("-s")
            .arg(pkg)
            .execution_mode(command::ExecutionMode::StatusOnly)
            .execute()
            .map(|output| output.exit_code)?;

        if status != 0 {
            command::CommandExecutor::new()
                .command("apt-get")
                .arg("install")
                .arg("-y")
                .arg(pkg)
                .execute_privileged()?;
            installed_packages.push(pkg.to_string());
        }
    }

    for ppa in &normalized_ppas {
        command::CommandExecutor::new()
            .command("add-apt-repository")
            .arg("-y")
            .arg(ppa)
            .execute_privileged()?;
        installed_ppas.push(ppa.clone());
    }

    command::CommandExecutor::new()
        .command("apt-get")
        .arg("update")
        .arg("-y")
        .execute_privileged()?;

    Ok((installed_ppas, installed_packages))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ppa_support_packages() {
        assert!(PPA_SUPPORT_PACKAGES.contains(&"software-properties-common"));
    }

    #[test]
    fn test_ppa_support_packages_debian() {
        assert!(PPA_SUPPORT_PACKAGES_DEBIAN.contains(&"python3-launchpadlib"));
    }

    #[test]
    fn test_install_function_signature() {
        let packages = vec!["test".to_string()];
        let result = install(&packages, None, false);
        let _ = result;
    }
}
