use anyhow::{Context, Result};

fn brew_cmd() -> std::process::Command {
    std::process::Command::new("brew")
}

fn brew_install_packages(packages: &[String]) -> std::process::Command {
    let mut cmd = std::process::Command::new("brew");
    cmd.arg("install");
    for pkg in packages {
        cmd.arg(pkg);
    }
    cmd
}

fn brew_update() -> std::process::Command {
    let mut cmd = std::process::Command::new("brew");
    cmd.arg("update");
    cmd
}

fn brew_cleanup() -> std::process::Command {
    let mut cmd = std::process::Command::new("brew");
    cmd.arg("cleanup");
    cmd
}

fn brew_backup_cache(cache_backup: &std::path::Path) -> Result<()> {
    let cache_dir = dirs::home_dir()
        .context("Failed to get home directory")?
        .join("Library/Caches/Homebrew");
    if cache_dir.exists() {
        std::fs::create_dir_all(cache_backup).context("Failed to create backup cache directory")?;
        std::process::Command::new("cp")
            .arg("-p")
            .arg("-R")
            .arg(&cache_dir)
            .arg(cache_backup)
            .status()
            .context("Failed to copy Homebrew cache")?;
    }
    Ok(())
}

/// Install packages using Homebrew
pub fn install(packages: &[String]) -> Result<()> {
    let status = brew_cmd()
        .arg("--version")
        .status()
        .context("Failed to run 'brew --version'")?;

    if !status.success() {
        anyhow::bail!("Homebrew not found");
    }

    let temp_dir = tempfile::tempdir().context("Failed to create temp directory")?;
    let cache_backup = temp_dir.path().join("brew_cache");
    brew_backup_cache(&cache_backup).context("Failed to backup Homebrew cache")?;

    brew_update()
        .status()
        .context("Failed to update Homebrew")?;
    brew_install_packages(packages)
        .status()
        .context("Failed to install packages with Homebrew")?;
    brew_cleanup()
        .status()
        .context("Failed to clean up Homebrew cache")?;

    if cache_backup.exists() {
        fs_extra::dir::copy(
            cache_backup,
            dirs::home_dir().unwrap().join("Library/Caches/Homebrew"),
            &fs_extra::dir::CopyOptions::new()
                .overwrite(true)
                .copy_inside(true),
        )
        .context("Failed to restore Homebrew cache")?;
    }

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
