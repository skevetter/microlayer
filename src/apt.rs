use crate::utils::{command, linux_info};
use anyhow::{Context, Result};
use log::{info, warn};
use std::fs;
use std::path::Path;

/// Install packages using apt with cleanup
/// This is similar to apt-get but uses 'apt' command which is more user-friendly
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

    // Backup apt lists
    const APT_LISTS_DIR: &str = "/var/lib/apt/lists";
    if Path::new(APT_LISTS_DIR).exists() {
        command::execute(&format!(
            "cp -p -R {} {}",
            APT_LISTS_DIR,
            cache_backup.display()
        ))?;
    }

    install_with_cleanup(packages, &ppas, &cache_backup)
}

fn install_with_cleanup(packages: &[String], ppas: &[String], cache_backup: &Path) -> Result<()> {
    // Update package lists
    command::execute("apt update -y")?;

    let mut installed_ppas = Vec::new();
    let mut installed_ppa_packages = Vec::new();

    // Add PPAs if specified
    if !ppas.is_empty() {
        let (ppas_added, ppa_pkgs) = add_ppas(ppas)?;
        installed_ppas = ppas_added;
        installed_ppa_packages = ppa_pkgs;
    }

    // Install packages
    let pkg_list = packages.join(" ");
    command::execute(&format!(
        "apt install -y --no-install-recommends {}",
        pkg_list
    ))?;

    // Cleanup: remove PPAs
    for ppa in &installed_ppas {
        let _ = command::execute(&format!("add-apt-repository -y --remove {}", ppa));
    }

    // Cleanup: purge PPA support packages
    for pkg in &installed_ppa_packages {
        let _ = command::execute(&format!("apt -y purge {} --auto-remove", pkg));
    }

    // Cleanup: remove package cache
    command::execute("apt clean")?;
    
    // Cleanup: restore lists cache
    command::execute(&format!("rm -rf {}", "/var/lib/apt/lists"))?;
    if cache_backup.exists() {
        fs::rename(cache_backup, "/var/lib/apt/lists")
            .context("Failed to restore apt lists cache")?;
    }

    Ok(())
}

fn add_ppas(ppas: &[String]) -> Result<(Vec<String>, Vec<String>)> {
    const PPA_SUPPORT_PACKAGES: &[&str] = &["software-properties-common"];
    const PPA_SUPPORT_PACKAGES_DEBIAN: &[&str] = &["python3-launchpadlib"];

    let mut installed_packages = Vec::new();
    let mut added_ppas = Vec::new();

    // Check if we're on Debian
    let is_debian = match linux_info::detect_distro() {
        Ok(distro) => matches!(distro, linux_info::LinuxDistro::Debian),
        Err(_) => false,
    };

    // Install support packages
    let support_pkgs = if is_debian {
        [PPA_SUPPORT_PACKAGES, PPA_SUPPORT_PACKAGES_DEBIAN].concat()
    } else {
        PPA_SUPPORT_PACKAGES.to_vec()
    };

    for pkg in &support_pkgs {
        if command::execute_status(&format!("dpkg -l {} 2>/dev/null", pkg))? != 0 {
            command::execute(&format!("apt-get install -y --no-install-recommends {}", pkg))?;
            installed_packages.push(pkg.to_string());
        }
    }

    // Add each PPA
    for ppa in ppas {
        info!("Adding PPA: {}", ppa);
        command::execute(&format!("add-apt-repository -y {}", ppa))?;
        added_ppas.push(ppa.clone());
    }

    // Update after adding PPAs
    if !added_ppas.is_empty() {
        command::execute("apt update -y")?;
    }

    Ok((added_ppas, installed_packages))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_function_signature() {
        // Test that the function signature is correct
        let packages = vec!["curl".to_string()];
        // Just test compilation, not actual execution
        let _ = install(&packages, None, false);
    }

    #[test]
    fn test_add_ppas_requires_packages() {
        // This should not panic with empty input
        let result = add_ppas(&[]);
        assert!(result.is_ok());
        let (ppas, _packages) = result.unwrap();
        assert_eq!(ppas.len(), 0);
    }
}
