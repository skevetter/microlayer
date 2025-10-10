use crate::apt_get;
use crate::utils::{command, linux_info};
use anyhow::{Context, Result};
use log::{info, warn};
use std::fs;
use std::path::Path;

const APT_LISTS_DIR: &str = "/var/lib/apt/lists";

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
        .command("apt")
        .arg("update")
        .arg("-y")
        .execute_privileged()
        .context("Failed to update apt repositories")?;

    let mut installed_ppas = Vec::new();
    let mut installed_ppa_packages = Vec::new();

    if !ppas.is_empty() {
        let (ppas_added, ppa_pkgs) = apt_get::add_ppas(ppas)?;
        installed_ppas.extend(ppas_added);
        installed_ppa_packages.extend(ppa_pkgs);
    }

    let pkgs = packages.iter().map(|s| s.as_str()).collect::<Vec<&str>>();

    command::CommandExecutor::new()
        .command("apt")
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
            .command("apt")
            .arg("-y")
            .arg("purge")
            .arg(pkg)
            .arg("--auto-remove")
            .execute_privileged()?;
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
