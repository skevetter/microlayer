use anyhow::{Context, Result};
use log::debug;

/// Install packages using Homebrew
pub fn install(packages: &[String]) -> Result<()> {
    anyhow::ensure!(
        which::which("brew").is_ok(),
        "Homebrew not installed or not in PATH"
    );

    debug!("Updating Homebrew");
    brew_update()
        .output()
        .map(|o| debug!("Brew update output: {:?}", o))
        .context("Failed to update Homebrew")?;

    debug!("Installing Homebrew packages: {:?}", packages);
    brew_install(packages)
        .output()
        .map(|o| debug!("Brew install output: {:?}", o))
        .context("Failed to install Homebrew packages")?;

    debug!("Cleaning up Homebrew cache");
    brew_cleanup()
        .output()
        .map(|o| debug!("Brew cleanup output: {:?}", o))
        .context("Failed to clean up Homebrew cache")?;

    Ok(())
}

fn brew() -> std::process::Command {
    std::process::Command::new("brew")
}

fn brew_install(packages: &[String]) -> std::process::Command {
    let mut cmd = brew();
    cmd.arg("install").args(packages);
    cmd
}

fn brew_update() -> std::process::Command {
    let mut cmd = brew();
    cmd.arg("update");
    cmd
}

fn brew_cleanup() -> std::process::Command {
    let mut cmd = brew();
    cmd.arg("cleanup");
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_install_function_exists() {
        let packages = vec!["nonexistent-package-12345".to_string()];
        let result = install(&packages);
        let _ = result;
    }
}
