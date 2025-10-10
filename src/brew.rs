use crate::utils::command;
use anyhow::{Context, Result};
use log::info;

/// Install packages using Homebrew
pub fn install(packages: &[String]) -> Result<()> {
    if !command::CommandBuilder::new("brew")
        .arg("--version")
        .execute_status()
        .map(|status| status.is_success())?
    {
        anyhow::bail!(
            "Homebrew is not available. Install Homebrew from https://brew.sh\n\
             Installation: /bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
        );
    }

    command::CommandBuilder::new("brew")
        .arg("update")
        .execute()
        .context("Failed to update Homebrew")?;

    info!("Installing packages with brew: {}", packages.join(", "));
    let pkgs = packages.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
    command::CommandBuilder::new("brew")
        .arg("install")
        .args(&pkgs)
        .execute()
        .context("Failed to install packages with brew")?;

    info!("Cleaning up Homebrew cache...");
    command::CommandBuilder::new("brew")
        .arg("cleanup")
        .execute()
        .context("Failed to clean up Homebrew cache")?;

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
