use crate::utils::{command, linux_info};
use anyhow::{Context, Result};
use log::{info, warn};
use std::fs;
use std::path::Path;

const PPA_SUPPORT_PACKAGES: &[&str] = &["software-properties-common"];
const PPA_SUPPORT_PACKAGES_DEBIAN: &[&str] = &["python3-launchpadlib"];

/// Install packages using apt-get with optional PPAs
pub fn install(
    packages: &[String],
    ppas: Option<&[String]>,
    force_ppas_on_non_ubuntu: bool,
) -> Result<()> {
    anyhow::ensure!(
        linux_info::is_debian_like(),
        "apt-get should be used on Debian-like distributions (Debian, Ubuntu, etc.)"
    );

    let mut ppas = ppas.map(|p| p.to_vec()).unwrap_or_default();

    if !ppas.is_empty() && !linux_info::is_ubuntu() && !force_ppas_on_non_ubuntu {
        warn!("PPAs are ignored on non-Ubuntu distros!");
        info!("Use --force-ppas-on-non-ubuntu to include them anyway.");
        ppas.clear();
    }

    let temp_dir = tempfile::tempdir().context("Failed to create temp directory")?;
    let cache_backup = temp_dir.path().join("lists");

    if Path::new("/var/lib/apt/lists").exists() {
        command::execute(&format!(
            "cp -p -R /var/lib/apt/lists {}",
            cache_backup.display()
        ))?;
    }

    install_with_cleanup(packages, &ppas, &cache_backup)
}

fn install_with_cleanup(packages: &[String], ppas: &[String], cache_backup: &Path) -> Result<()> {
    command::execute("apt-get update -y")?;

    let mut installed_ppas = Vec::new();
    let mut installed_ppa_packages = Vec::new();

    if !ppas.is_empty() {
        let (ppas_added, ppa_pkgs) = add_ppas(ppas)?;
        installed_ppas = ppas_added;
        installed_ppa_packages = ppa_pkgs;
    }

    let pkg_list = packages.join(" ");
    command::execute(&format!(
        "apt-get install -y --no-install-recommends {}",
        pkg_list
    ))?;

    for ppa in &installed_ppas {
        let _ = command::execute(&format!("add-apt-repository -y --remove {}", ppa));
    }

    for pkg in &installed_ppa_packages {
        let _ = command::execute(&format!("apt-get -y purge {} --auto-remove", pkg));
    }

    command::execute("apt-get clean")?;
    command::execute("rm -rf /var/lib/apt/lists")?;
    if cache_backup.exists() {
        fs::rename(cache_backup, "/var/lib/apt/lists")
            .context("Failed to restore apt lists cache")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ppa_support_packages() {
        assert!(PPA_SUPPORT_PACKAGES.contains(&"software-properties-common"));
    }

    #[test]
    fn test_ppa_support_packages_debian() {
        assert!(PPA_SUPPORT_PACKAGES_DEBIAN.contains(&"python3-launchpadlib"));
    }

    #[test]
    fn test_install_function_signature() {
        let packages = vec!["test".to_string()];
        let result = install(&packages, None, false);
        let _ = result;
    }
}

fn add_ppas(ppas: &[String]) -> Result<(Vec<String>, Vec<String>)> {
    let mut installed_ppas = Vec::new();
    let mut installed_packages = Vec::new();

    let normalized_ppas: Vec<String> = ppas
        .iter()
        .map(|ppa| {
            if ppa.starts_with("ppa:") {
                ppa.clone()
            } else {
                format!("ppa:{}", ppa)
            }
        })
        .collect();

    let required_packages: Vec<&str> = if linux_info::is_ubuntu() {
        PPA_SUPPORT_PACKAGES.to_vec()
    } else {
        PPA_SUPPORT_PACKAGES
            .iter()
            .chain(PPA_SUPPORT_PACKAGES_DEBIAN.iter())
            .copied()
            .collect()
    };

    for pkg in required_packages {
        let status = command::execute_status(&format!("dpkg -s {}", pkg))?;
        if status != 0 {
            command::execute(&format!("apt-get install -y {}", pkg))?;
            installed_packages.push(pkg.to_string());
        }
    }

    for ppa in &normalized_ppas {
        command::execute(&format!("add-apt-repository -y {}", ppa))?;
        installed_ppas.push(ppa.clone());
    }

    command::execute("apt-get update -y")?;

    Ok((installed_ppas, installed_packages))
}
