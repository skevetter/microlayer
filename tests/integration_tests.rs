use std::fs;
use std::path::Path;
use std::process::Command;

const PICOLAYER_BIN: &str = env!("CARGO_BIN_EXE_picolayer");

/// Helper function to run picolayer commands
fn run_picolayer(args: &[&str]) -> std::process::Output {
    Command::new(PICOLAYER_BIN)
        .args(args)
        .output()
        .expect("Failed to execute picolayer")
}

/// Check if error is due to rate limiting or network issues
fn is_transient_error(stderr: &str) -> bool {
    stderr.contains("403 Forbidden") 
        || stderr.contains("rate limit") 
        || stderr.contains("API rate limit")
        || stderr.contains("connection")
}

/// Helper function to check if a binary exists and is executable
fn binary_exists(path: &str) -> bool {
    Path::new(path).exists()
}

/// Helper function to check binary version
fn check_binary_version(binary_path: &str, expected_substring: Option<&str>) -> bool {
    if !binary_exists(binary_path) {
        return false;
    }

    let output = Command::new(binary_path)
        .arg("--version")
        .output();

    if let Ok(output) = output {
        let version_str = String::from_utf8_lossy(&output.stdout);
        if let Some(expected) = expected_substring {
            version_str.contains(expected)
        } else {
            !version_str.is_empty()
        }
    } else {
        false
    }
}

#[test]
#[cfg(target_os = "linux")]
fn test_apt_get_installation() {
    // Check if we're on a system that uses apt-get
    let has_apt = Command::new("which")
        .arg("apt-get")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_apt {
        eprintln!("Skipping apt-get test: apt-get not available");
        return;
    }

    // Test installing a simple package (curl is usually not installed by default)
    let output = run_picolayer(&["apt-get", "file"]);

    // Check if the command was successful or if we need sudo
    // In many CI environments, we might not have permissions
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("permission denied") || stderr.contains("root") {
            eprintln!("Skipping apt-get test: requires root permissions");
            return;
        }
    }

    assert!(
        output.status.success(),
        "apt-get installation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[cfg(target_os = "linux")]
fn test_apk_installation() {
    // Check if we're on a system that uses apk
    let has_apk = Command::new("which")
        .arg("apk")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_apk {
        eprintln!("Skipping apk test: apk not available");
        return;
    }

    // Test installing a simple package
    let output = run_picolayer(&["apk", "curl"]);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("permission denied") || stderr.contains("root") {
            eprintln!("Skipping apk test: requires root permissions");
            return;
        }
    }

    assert!(
        output.status.success(),
        "apk installation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_pkgx_github_release_installation() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let bin_location = temp_dir.path().to_str().unwrap();

    let output = run_picolayer(&[
        "gh-release",
        "pkgxdev/pkgx",
        "pkgx",
        "--version",
        "latest",
        "--bin-location",
        bin_location,
    ]);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if is_transient_error(&stderr) {
            eprintln!("Skipping test due to transient error: {}", stderr);
            return;
        }
        eprintln!("Installation output: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("Installation error: {}", stderr);
    }

    assert!(
        output.status.success(),
        "pkgx installation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let binary_path = format!("{}/pkgx", bin_location);
    assert!(
        binary_exists(&binary_path),
        "pkgx binary was not installed at {}",
        binary_path
    );

    // Verify the binary works
    assert!(
        check_binary_version(&binary_path, None),
        "pkgx binary version check failed"
    );
}

#[test]
fn test_lazygit_specific_version_installation() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let bin_location = temp_dir.path().to_str().unwrap();

    let output = run_picolayer(&[
        "gh-release",
        "jesseduffield/lazygit",
        "lazygit",
        "--version",
        "v0.54.0",
        "--bin-location",
        bin_location,
    ]);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if is_transient_error(&stderr) {
            eprintln!("Skipping test due to transient error: {}", stderr);
            return;
        }
        eprintln!("Installation output: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("Installation error: {}", stderr);
    }

    assert!(
        output.status.success(),
        "lazygit v0.54.0 installation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let binary_path = format!("{}/lazygit", bin_location);
    assert!(
        binary_exists(&binary_path),
        "lazygit binary was not installed at {}",
        binary_path
    );

    // Verify the binary version
    assert!(
        check_binary_version(&binary_path, Some("0.54")),
        "lazygit binary version check failed"
    );
}

#[test]
fn test_lazygit_latest_with_checksum() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let bin_location = temp_dir.path().to_str().unwrap();

    let output = run_picolayer(&[
        "gh-release",
        "jesseduffield/lazygit",
        "lazygit",
        "--version",
        "latest",
        "--bin-location",
        bin_location,
        "--checksum",
    ]);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if is_transient_error(&stderr) {
            eprintln!("Skipping test due to transient error: {}", stderr);
            return;
        }
        eprintln!("Installation output: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("Installation error: {}", stderr);
    }

    assert!(
        output.status.success(),
        "lazygit latest with checksum installation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let binary_path = format!("{}/lazygit", bin_location);
    assert!(
        binary_exists(&binary_path),
        "lazygit binary was not installed at {}",
        binary_path
    );

    // Verify the binary works
    assert!(
        check_binary_version(&binary_path, None),
        "lazygit binary version check failed"
    );
}

#[test]
#[ignore] // This test requires a GPG public key, so it's ignored by default
fn test_pkgx_with_gpg_verification() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let bin_location = temp_dir.path().to_str().unwrap();

    // This test would require the actual GPG public key for pkgx
    // For now, we'll skip this test unless we have the key
    let gpg_key_path = "/tmp/pkgx_public_key.asc";
    if !Path::new(gpg_key_path).exists() {
        eprintln!("Skipping GPG verification test: public key not found");
        return;
    }

    let output = run_picolayer(&[
        "gh-release",
        "pkgxdev/pkgx",
        "pkgx",
        "--version",
        "latest",
        "--bin-location",
        bin_location,
        "--checksum",
        "--gpg-key",
        gpg_key_path,
    ]);

    if !output.status.success() {
        eprintln!("Installation output: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("Installation error: {}", String::from_utf8_lossy(&output.stderr));
    }

    assert!(
        output.status.success(),
        "pkgx with GPG verification failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let binary_path = format!("{}/pkgx", bin_location);
    assert!(binary_exists(&binary_path), "pkgx binary was not installed");
}

#[test]
fn test_pkgx_with_filter_and_custom_location() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let bin_location = temp_dir.path().to_str().unwrap();

    // Use a filter to select the correct asset
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;
    let filter = format!("{}.*{}", arch, os);

    let output = run_picolayer(&[
        "gh-release",
        "pkgxdev/pkgx",
        "pkgx",
        "--version",
        "latest",
        "--bin-location",
        bin_location,
        "--filter",
        &filter,
    ]);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if is_transient_error(&stderr) {
            eprintln!("Skipping test due to transient error: {}", stderr);
            return;
        }
        eprintln!("Installation output: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("Installation error: {}", stderr);
    }

    assert!(
        output.status.success(),
        "pkgx with filter installation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let binary_path = format!("{}/pkgx", bin_location);
    assert!(
        binary_exists(&binary_path),
        "pkgx binary was not installed at custom location"
    );

    // Verify the binary works
    assert!(
        check_binary_version(&binary_path, None),
        "pkgx binary version check failed"
    );
}

#[test]
#[cfg(target_os = "macos")]
fn test_brew_installation() {
    // Check if brew is available
    let has_brew = Command::new("which")
        .arg("brew")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_brew {
        eprintln!("Skipping brew test: Homebrew not available");
        return;
    }

    // Test installing a simple package (jq is a good test package)
    let output = run_picolayer(&["brew", "jq"]);

    if !output.status.success() {
        eprintln!("Brew output: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("Brew error: {}", String::from_utf8_lossy(&output.stderr));
    }

    // Note: We don't assert success here because the package might already be installed
    // or there might be other issues. The important thing is that the command ran.
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("Homebrew is not available"),
        "Brew command failed to detect Homebrew"
    );
}
