use anyhow::{Context, Result};

struct Brew {}

impl Brew {
    fn brew() -> std::process::Command {
        std::process::Command::new("brew")
    }

    fn install_packages(packages: &[String]) -> std::process::Command {
        let mut cmd = std::process::Command::new("brew");
        cmd.arg("install");
        for pkg in packages {
            cmd.arg(pkg);
        }
        cmd
    }

    fn update() -> std::process::Command {
        let mut cmd = std::process::Command::new("brew");
        cmd.arg("update");
        cmd
    }

    fn cleanup() -> std::process::Command {
        let mut cmd = std::process::Command::new("brew");
        cmd.arg("cleanup");
        cmd
    }

    fn rm_brew_cache() {
        let _ = std::fs::remove_dir_all("~/Library/Caches/Homebrew");
    }
}

/// Install packages using Homebrew
pub fn install(packages: &[String]) -> Result<()> {
    let status = Brew::brew()
        .arg("--version")
        .status()
        .context("Failed to run 'brew --version'")?;

    if !status.success() {
        anyhow::bail!("Homebrew not found");
    }
    Brew::update()
        .status()
        .context("Failed to update Homebrew")?;
    Brew::install_packages(packages)
        .status()
        .context("Failed to install packages with Homebrew")?;
    Brew::cleanup()
        .status()
        .context("Failed to clean up Homebrew cache")?;

    Brew::rm_brew_cache();

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
