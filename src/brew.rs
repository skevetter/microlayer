use anyhow::{Context, Result};
use log::info;
use std::process::{Command, Stdio};

/// Install packages using Homebrew
pub fn install(packages: &[String]) -> Result<()> {
    let brew_available = Command::new("brew")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !brew_available {
        anyhow::bail!(
            "Homebrew is not available. Install Homebrew from https://brew.sh\n\
             Installation: /bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
        );
    }

    info!("Installing packages with brew: {}", packages.join(", "));

    info!("Updating Homebrew...");
    let status = Command::new("brew")
        .arg("update")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to update Homebrew")?;

    if !status.success() {
        anyhow::bail!(
            "brew update failed with exit code: {}",
            status.code().unwrap_or(-1)
        );
    }

    info!("Installing packages...");
    let status = Command::new("brew")
        .arg("install")
        .args(packages)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to install packages")?;

    if !status.success() {
        anyhow::bail!(
            "brew install failed with exit code: {}",
            status.code().unwrap_or(-1)
        );
    }

    info!("Cleaning up Homebrew cache...");
    let _ = Command::new("brew")
        .arg("cleanup")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    info!("Installation complete!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_function_exists() {
        let packages = vec!["nonexistent-package-12345".to_string()];
        let result = install(&packages);
        let _ = result;
    }
}
