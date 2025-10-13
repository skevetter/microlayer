use crate::installers::apt_get;
use crate::utils;
use anyhow::{Context, Result};
use log::debug;

/// Install packages using aptitude
pub fn install(packages: &[String]) -> Result<()> {
    anyhow::ensure!(
        utils::os_detect::is_debian_like(),
        "aptitude should be used on Debian-like distributions (Debian, Ubuntu, etc.)"
    );

    debug!("Updating aptitude repositories");
    aptitude_update()
        .output()
        .map(|o| debug!("Aptitude update output: {:?}", o))
        .context("Failed to update aptitude repositories")?;

    debug!("Installing aptitude");
    install_aptitude().context("Failed to install aptitude")?;

    debug!("Installing aptitude packages: {:?}", packages);
    aptitude_install(packages)
        .output()
        .map(|o| debug!("Aptitude install output: {:?}", o))
        .context("Failed to install packages with aptitude")?;

    debug!("Cleaning aptitude cache");
    aptitude_clean()
        .status()
        .context("Failed to clean aptitude cache")?;

    Ok(())
}

/// Installs the aptitude package using apt-get if not already installed
fn install_aptitude() -> Result<()> {
    if which::which("aptitude").is_err() {
        apt_get::install(&["aptitude".to_string()], Some(&[]), false)
            .context("Failed to install aptitude")?;
    }
    Ok(())
}

fn aptitude() -> std::process::Command {
    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("aptitude");
    cmd
}

fn aptitude_update() -> std::process::Command {
    let mut cmd = aptitude();
    cmd.arg("update");
    cmd
}

fn aptitude_clean() -> std::process::Command {
    let mut cmd = aptitude();
    cmd.arg("clean");
    cmd
}

fn aptitude_install(packages: &[String]) -> std::process::Command {
    let mut cmd = aptitude();
    cmd.arg("install")
        .arg("-y")
        .arg("--without-recommends")
        .args(packages);
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_aptitude() {
        let packages = vec!["curl".to_string()];
        let _ = aptitude_install(&packages);
    }
}
