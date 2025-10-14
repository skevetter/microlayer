use crate::installers::apt_get;
use crate::utils;
use anyhow::{Context, Result};
use log::{debug, info, warn};

/// Install packages using apt
pub fn install(
    packages: &[String],
    ppas: Option<&[String]>,
    force_ppas_on_non_ubuntu: bool,
) -> Result<()> {
    anyhow::ensure!(
        utils::os::is_debian_like() && which::which("apt").is_ok(),
        "apt should be used on Debian-like distributions (Debian, Ubuntu, etc.)"
    );

    let mut ppas = ppas.map(|p| p.to_vec()).unwrap_or_default();
    if !ppas.is_empty() && !utils::os::is_ubuntu() && !force_ppas_on_non_ubuntu {
        warn!("PPAs are ignored on non-Ubuntu distros!");
        info!("Use --force-ppas-on-non-ubuntu to include them anyway.");
        ppas.clear();
    }

    debug!("Updating apt repositories");
    apt_update()
        .output()
        .map(|o| debug!("Apt update output: {:?}", o))
        .context("Failed to update repositories")?;

    let mut installed_ppas = Vec::new();
    let mut installed_ppa_packages = Vec::new();

    if !ppas.is_empty() {
        let (ppas_added, ppa_pkgs) = apt_get::add_ppas(&ppas)?;
        installed_ppas.extend(ppas_added);
        installed_ppa_packages.extend(ppa_pkgs);
    }

    info!("Installing apt packages: {:?}", packages);
    apt_install(packages)
        .output()
        .map(|o| debug!("Apt install output: {:?}", o))
        .context("Failed to install apt packages")?;

    info!("Removing added PPAs and packages installed for PPA support");
    apt_remove_ppas(&installed_ppas)
        .output()
        .map(|o| debug!("Apt remove PPAs output: {:?}", o))
        .context("Failed to remove PPAs")?;

    info!("Purging packages installed for PPA support");
    apt_purge(installed_ppa_packages.as_slice())
        .output()
        .map(|o| debug!("Apt purge output: {:?}", o))
        .context("Failed to purge package installed for PPA support")?;

    info!("Cleaning up apt cache");
    apt_clean()
        .output()
        .map(|o| debug!("Apt clean output: {:?}", o))
        .context("Failed to clean apt cache")?;

    Ok(())
}

fn apt() -> std::process::Command {
    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("apt");
    cmd
}

fn apt_install(packages: &[String]) -> std::process::Command {
    let mut cmd = apt();
    cmd.arg("install")
        .arg("-y")
        .arg("--no-install-recommends")
        .args(packages);
    cmd
}

fn apt_update() -> std::process::Command {
    let mut cmd = apt();
    cmd.arg("update").arg("-y");
    cmd
}

fn apt_purge(packages: &[String]) -> std::process::Command {
    let mut cmd = apt();
    cmd.arg("purge")
        .arg("-y")
        .arg("--auto-remove")
        .args(packages);
    cmd
}

fn apt_clean() -> std::process::Command {
    let mut cmd = apt();
    cmd.arg("clean");
    cmd
}

fn apt_remove_ppas(ppas: &[String]) -> std::process::Command {
    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("add-apt-repository")
        .arg("-y")
        .arg("--remove")
        .args(ppas);
    cmd
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
