#[cfg(target_os = "linux")]
use common::run_picolayer;
#[cfg(target_os = "linux")]
use serial_test::serial;
#[cfg(target_os = "linux")]
use std::fs;
#[cfg(target_os = "linux")]
use std::path::Path;

/// Helper function to compute directory hash for comparison
#[cfg(target_os = "linux")]
fn compute_simple_hash(dir_path: &Path) -> String {
    use sha2::{Digest, Sha256};
    use walkdir::WalkDir;

    let mut hasher = Sha256::new();
    let mut entries: Vec<_> = WalkDir::new(dir_path)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|e| e.ok())
        .collect();

    entries.sort_by(|a, b| a.path().cmp(b.path()));

    for entry in entries {
        let path = entry.path();
        if let Ok(rel_path) = path.strip_prefix(dir_path) {
            hasher.update(rel_path.to_string_lossy().as_bytes());
        }
        if path.is_file() {
            if let Ok(contents) = fs::read(path) {
                hasher.update(&contents);
            }
        }
    }

    format!("{:x}", hasher.finalize())
}

#[test]
#[serial]
#[cfg(target_os = "linux")]
fn test_apt_get_cache_restoration() {
    // Ensure apt lists exist
    std::process::Command::new("sudo")
        .arg("apt-get")
        .arg("update")
        .status()
        .expect("Failed to run apt-get update");

    let lists_path = Path::new("/var/lib/apt/lists");
    assert!(lists_path.exists(), "/var/lib/apt/lists does not exist");

    // Compute hash of the current state
    let initial_hash = compute_simple_hash(lists_path);

    // Run picolayer to install a package
    let output = run_picolayer(&["apt-get", "curl"]);
    assert!(
        output.status.success(),
        "apt-get installation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify cache was restored by comparing hash
    let final_hash = compute_simple_hash(lists_path);
    assert_eq!(
        initial_hash, final_hash,
        "Cache was not restored to original state"
    );
}

#[test]
#[serial]
#[cfg(target_os = "linux")]
fn test_apt_cache_restoration() {
    // Ensure apt lists exist
    std::process::Command::new("sudo")
        .arg("apt")
        .arg("update")
        .status()
        .expect("Failed to run apt update");

    let lists_path = Path::new("/var/lib/apt/lists");
    assert!(lists_path.exists(), "/var/lib/apt/lists does not exist");

    // Compute hash of the current state
    let initial_hash = compute_simple_hash(lists_path);

    // Run picolayer to install a package
    let output = run_picolayer(&["apt", "curl"]);
    assert!(
        output.status.success(),
        "apt installation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify cache was restored by comparing hash
    let final_hash = compute_simple_hash(lists_path);
    assert_eq!(
        initial_hash, final_hash,
        "Cache was not restored to original state"
    );
}

#[test]
#[serial]
#[cfg(target_os = "linux")]
fn test_temp_files_cleanup() {
    let temp_dir = Path::new("/tmp/picolayer");

    // Record initial state of temp directory
    let initial_files: Vec<_> = if temp_dir.exists() {
        fs::read_dir(temp_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name())
            .collect()
    } else {
        vec![]
    };

    // Run picolayer
    let output = run_picolayer(&["apt-get", "file"]);
    assert!(
        output.status.success(),
        "Installation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Check that no new persistent files were left behind
    let final_files: Vec<_> = if temp_dir.exists() {
        fs::read_dir(temp_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name())
            .collect()
    } else {
        vec![]
    };

    // Filter out lock files which are expected to exist
    let new_files: Vec<_> = final_files
        .iter()
        .filter(|f| {
            !initial_files.contains(f)
                && !f.to_string_lossy().contains("lock")
                && !f.to_string_lossy().starts_with("picolayer_")
        })
        .collect();

    assert!(
        new_files.is_empty(),
        "Unexpected temporary files left behind: {:?}",
        new_files
    );
}

#[test]
#[serial]
#[cfg(target_os = "linux")]
fn test_apk_cache_restoration() {
    // Skip if not on Alpine
    if !std::path::Path::new("/etc/alpine-release").exists() {
        return;
    }

    let cache_dir = Path::new("/var/cache/apk");

    // Record initial state
    let initial_exists = cache_dir.exists();
    let initial_hash = if initial_exists {
        Some(compute_simple_hash(cache_dir))
    } else {
        None
    };

    // Run picolayer to install a package
    let output = run_picolayer(&["apk", "curl"]);
    assert!(
        output.status.success(),
        "apk installation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify cache state
    let final_exists = cache_dir.exists();
    assert_eq!(
        initial_exists, final_exists,
        "Cache directory existence changed"
    );

    if let Some(initial) = initial_hash {
        let final_hash = compute_simple_hash(cache_dir);
        assert_eq!(
            initial, final_hash,
            "Cache was not restored to original state"
        );
    }
}
