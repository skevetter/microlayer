use crate::utils;
use anyhow::{Context, Result};
use log::debug;

/// Install packages using apk
pub fn install(packages: &[String]) -> Result<()> {
    anyhow::ensure!(
        utils::os::is_alpine(),
        "apk should be used on Alpine Linux distribution"
    );

    debug!("Updating apk repositories");
    apk_update()
        .output()
        .map(|o| debug!("Apk update output: {:?}", o))
        .context("Failed to update apk repositories")?;

    debug!("Installing apk packages: {:?}", packages);
    apk_add_pkgs(packages)
        .output()
        .map(|o| debug!("Apk add output: {:?}", o))
        .context("Failed to install apk packages")?;

    debug!("Cleaning up apk cache");
    apk_clean()
        .output()
        .map(|o| debug!("Apk clean output: {:?}", o))
        .context("Failed to clean apk cache")?;

    Ok(())
}

fn apk() -> std::process::Command {
    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("apk");
    cmd
}

fn apk_clean() -> std::process::Command {
    let mut cmd = apk();
    cmd.arg("cache").arg("clean");
    cmd
}

fn apk_update() -> std::process::Command {
    let mut cmd = apk();
    cmd.arg("update");
    cmd
}

fn apk_add_pkgs(packages: &[String]) -> std::process::Command {
    let mut cmd = apk();
    cmd.arg("add").arg("--no-cache").args(packages);
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_apk() {
        let packages = vec!["test".to_string()];
        let result = install(&packages);
        let _ = result;
    }
}
