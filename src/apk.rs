use crate::utils::{command, linux_info};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

const APK_CACHE_DIR: &str = "/var/cache/apk";

/// Install packages using apk
pub fn install(packages: &[String]) -> Result<()> {
    anyhow::ensure!(
        linux_info::is_alpine(),
        "apk should be used on Alpine Linux distribution"
    );

    let temp_dir = tempfile::tempdir().context("Failed to create temp directory")?;
    let cache_backup = temp_dir.path().join("apk");

    if Path::new(APK_CACHE_DIR).exists() {
        fs_extra::copy_items(
            &[APK_CACHE_DIR],
            &cache_backup,
            &fs_extra::dir::CopyOptions::new(),
        )
        .context("Failed to copy apk cache")?;
        //recursive_copy(APK_CACHE_DIR, &cache_backup).context("Failed to copy apk cache")?;`
    }

    install_with_cleanup(packages, &cache_backup)
}

// fn recursive_copy(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
//     let src = src.as_ref();
//     let dst = dst.as_ref();

//     fs::create_dir_all(dst)?;

//     for entry in fs::read_dir(src)? {
//         let entry = entry?;
//         let path = entry.path();
//         let dest_path = dst.join(entry.file_name());
//         let ft = entry.file_type()?;

//         if ft.is_dir() {
//             recursive_copy(&path, &dest_path)?;
//         } else if ft.is_file() {
//             fs::copy(&path, &dest_path)?;
//         } else if ft.is_symlink() {
//             let target = fs::read_link(&path).context("reading symlink target");
//             if let Ok(target) = target {
//                 let _ = unix_fs::symlink(target, dest_path).context("creating symlink");
//             }
//         }
//     }

//     Ok(())
// }

fn install_with_cleanup(packages: &[String], cache_backup: &Path) -> Result<()> {
    let pkgs: Vec<&str> = packages.iter().map(|s| s.as_str()).collect();

    command::CommandExecutor::new()
        .command("apk")
        .arg("update")
        .execute_privileged()
        .context("Failed to update apk repositories")?;
    command::CommandExecutor::new()
        .command("apk")
        .arg("add")
        .arg("--no-cache")
        .args(&pkgs)
        .execute_privileged()
        .context("Failed to install apk packages")?;
    command::CommandExecutor::new()
        .command("rm")
        .arg("-rf")
        .arg(APK_CACHE_DIR)
        .execute_privileged()
        .context("Failed to remove apk cache")?;

    if cache_backup.exists() {
        fs::rename(cache_backup, APK_CACHE_DIR).context("Failed to restore apk cache")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_function_signature() {
        let packages = vec!["test".to_string()];
        let result = install(&packages);
        let _ = result;
    }
}
