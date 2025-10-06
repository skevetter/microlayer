use crate::common::{binary_exists, check_binary_version, is_transient_error, run_picolayer};

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
        eprintln!(
            "Installation output: {}",
            String::from_utf8_lossy(&output.stdout)
        );
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
        eprintln!(
            "Installation output: {}",
            String::from_utf8_lossy(&output.stdout)
        );
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
        eprintln!(
            "Installation output: {}",
            String::from_utf8_lossy(&output.stdout)
        );
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

    assert!(
        check_binary_version(&binary_path, None),
        "lazygit binary version check failed"
    );
}

#[test]
#[ignore]
fn test_pkgx_with_gpg_verification() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let bin_location = temp_dir.path().to_str().unwrap();

    let gpg_key_url = "https://dist.pkgx.dev/gpg-public.asc";

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
        gpg_key_url,
    ]);

    if !output.status.success() {
        eprintln!(
            "Installation output: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        eprintln!(
            "Installation error: {}",
            String::from_utf8_lossy(&output.stderr)
        );
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

    let arch = if std::env::consts::ARCH == "x86_64" {
        "x86-64"
    } else {
        std::env::consts::ARCH
    };
    let os = if std::env::consts::OS == "macos" {
        "darwin"
    } else {
        std::env::consts::OS
    };
    let filter = format!("{}.*{}", os, arch);
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
        eprintln!(
            "Installation output: {}",
            String::from_utf8_lossy(&output.stdout)
        );
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

    assert!(
        check_binary_version(&binary_path, None),
        "pkgx binary version check failed"
    );
}
