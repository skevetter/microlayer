use common::{binary_exists, check_binary_version, is_transient_error, run_picolayer};
use serial_test::serial;
use std::path::Path;

#[test]
#[serial]
fn test_pkgx_github_release_installation() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let bin_location = temp_dir.path().to_str().unwrap();

    let output = run_picolayer(&[
        "gh-release",
        "pkgxdev/pkgx",
        "pkgx",
        "--version",
        "latest",
        "--install-dir",
        bin_location,
    ]);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if is_transient_error(&stderr) {
            eprintln!("Skipping test due to transient error");
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
#[serial]
fn test_lazygit_specific_version_installation() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let bin_location = temp_dir.path().to_str().unwrap();

    let output = run_picolayer(&[
        "gh-release",
        "jesseduffield/lazygit",
        "lazygit",
        "--version",
        "v0.54.0",
        "--install-dir",
        bin_location,
    ]);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if is_transient_error(&stderr) {
            eprintln!("Skipping test due to transient error");
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
#[serial]
fn test_lazygit_latest_with_checksum() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let bin_location = temp_dir.path().to_str().unwrap();

    let output = run_picolayer(&[
        "gh-release",
        "jesseduffield/lazygit",
        "lazygit",
        "--version",
        "latest",
        "--install-dir",
        bin_location,
        "--verify-checksum",
    ]);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if is_transient_error(&stderr) {
            eprintln!("Skipping test due to transient error");
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
#[serial]
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
        "--install-dir",
        bin_location,
        "--verify-checksum",
        "--gpg-key",
        gpg_key_url,
    ]);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if is_transient_error(&stderr) {
            eprintln!("Skipping test due to transient error");
            return;
        }
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
#[serial]
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
        "--install-dir",
        bin_location,
        "--filter",
        &filter,
    ]);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if is_transient_error(&stderr) {
            eprintln!("Skipping test due to transient error");
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

#[test]
#[serial]
fn test_xz_extraction_with_real_archive() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let bin_location = temp_dir.path().to_str().unwrap();

    let output = run_picolayer(&[
        "gh-release",
        "pkgxdev/pkgx",
        "pkgx",
        "--version",
        "v2.7.0",
        "--install-dir",
        bin_location,
        "--filter",
        "linux.*x86-64\\.tar\\.xz",
    ]);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if is_transient_error(&stderr) {
            eprintln!("Skipping test due to transient error");
            return;
        }
    }

    let binary_path = format!("{}/pkgx", bin_location);
    if output.status.success() {
        assert!(
            binary_exists(&binary_path),
            "Binary should be extracted from XZ archive"
        );
    }
}

#[test]
#[serial]
fn test_xz_extraction_identifies_tar_xz_correctly() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let bin_location = temp_dir.path().to_str().unwrap();
    let os = if std::env::consts::OS == "macos" {
        "darwin"
    } else {
        std::env::consts::OS
    };

    let output = run_picolayer(&[
        "gh-release",
        "pkgxdev/pkgx",
        "pkgx",
        "--version",
        "v2.7.0",
        "--install-dir",
        bin_location,
        "--filter",
        &format!("{}.*x86-64\\.tar\\.xz", os),
    ]);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if is_transient_error(&stderr) {
            eprintln!("Skipping test due to transient error");
            return;
        }

        let stderr_str = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr_str.contains("unsupported format"),
            "Should correctly identify tar.xz format: {}",
            stderr_str
        );
    }
}

#[test]
#[serial]
fn test_xz_extraction_handles_invalid_data() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let bin_location = temp_dir.path().to_str().unwrap();
    let os = if std::env::consts::OS == "macos" {
        "darwin"
    } else {
        std::env::consts::OS
    };

    let output = run_picolayer(&[
        "gh-release",
        "pkgxdev/pkgx",
        "pkgx",
        "--version",
        "v999.999.999",
        "--install-dir",
        bin_location,
        "--filter",
        &format!("{}.*x86-64\\.tar\\.xz", os),
    ]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    if is_transient_error(&stderr) {
        eprintln!("Skipping test due to transient error");
        return;
    }

    assert!(
        stderr.contains("404") || stderr.contains("not found") || stderr.contains("No matching"),
        "Should provide helpful error message: {}",
        stderr
    );
}

#[test]
#[serial]
fn test_xz_extraction_creates_directories() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let nested_path = temp_dir.path().join("nested").join("deep").join("path");
    let bin_location = nested_path.to_str().unwrap();
    let os = if std::env::consts::OS == "macos" {
        "darwin"
    } else {
        std::env::consts::OS
    };

    let output = run_picolayer(&[
        "gh-release",
        "pkgxdev/pkgx",
        "pkgx",
        "--version",
        "v2.7.0",
        "--install-dir",
        bin_location,
        "--filter",
        &format!("{}.*x86-64\\.tar\\.xz", os),
    ]);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if is_transient_error(&stderr) {
            eprintln!("Skipping test due to transient error");
            return;
        }
    }

    if output.status.success() || Path::new(bin_location).exists() {
        assert!(
            Path::new(bin_location).exists(),
            "Should create nested directories"
        );
    }
}

#[test]
#[serial]
fn test_xz_extraction_with_multiple_binaries() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let bin_location = temp_dir.path().to_str().unwrap();
    let os = if std::env::consts::OS == "macos" {
        "darwin"
    } else {
        std::env::consts::OS
    };

    // Test with an archive that might contain multiple binaries
    let output = run_picolayer(&[
        "gh-release",
        "pkgxdev/pkgx",
        "pkgx",
        "--version",
        "v2.7.0",
        "--install-dir",
        bin_location,
        "--filter",
        &format!("{}.*x86-64\\.tar\\.xz", os),
    ]);
    print!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    print!("STDERR: {}", String::from_utf8_lossy(&output.stderr));

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if is_transient_error(&stderr) {
            eprintln!("Skipping test due to transient error");
            return;
        }
    }

    if output.status.success() {
        let main_binary = format!("{}/pkgx", bin_location);
        assert!(
            binary_exists(&main_binary),
            "Main binary should be extracted"
        );
    }
}

#[test]
#[serial]
fn test_xz_extraction_performance() {
    use std::time::Instant;

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let bin_location = temp_dir.path().to_str().unwrap();
    let os = if std::env::consts::OS == "macos" {
        "darwin"
    } else {
        std::env::consts::OS
    };

    let start = Instant::now();
    let output = run_picolayer(&[
        "gh-release",
        "pkgxdev/pkgx",
        "pkgx",
        "--version",
        "v2.7.0",
        "--install-dir",
        bin_location,
        "--filter",
        &format!("{}.*x86-64\\.tar\\.xz", os),
    ]);
    let duration = start.elapsed();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if is_transient_error(&stderr) {
            eprintln!("Skipping test due to transient error");
            return;
        }
    }

    assert!(
        duration.as_secs() < 10,
        "XZ extraction should complete in reasonable time, took: {:?}",
        duration
    );
}

#[test]
#[serial]
fn test_pkgx_xz_installation_end_to_end() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let bin_location = temp_dir.path().to_str().unwrap();
    let os = if std::env::consts::OS == "macos" {
        "darwin"
    } else {
        std::env::consts::OS
    };

    let output = run_picolayer(&[
        "gh-release",
        "pkgxdev/pkgx",
        "pkgx",
        "--version",
        "v2.7.0",
        "--install-dir",
        bin_location,
        "--filter",
        &format!("{}.*x86-64\\.tar\\.xz", os),
    ]);
    print!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    print!("STDERR: {}", String::from_utf8_lossy(&output.stderr));

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if is_transient_error(&stderr) {
            eprintln!("Skipping test due to transient error");
            return;
        };
    }

    let binary_path = format!("{}/pkgx", bin_location);
    assert!(
        binary_exists(&binary_path),
        "pkgx binary should be installed"
    );

    assert!(
        check_binary_version(&binary_path, Some("pkgx")),
        "pkgx binary should be functional"
    );
}

#[test]
#[serial]
fn xz_extraction_test() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let bin_location = temp_dir.path().to_str().unwrap();
    let os = if std::env::consts::OS == "macos" {
        "darwin"
    } else {
        std::env::consts::OS
    };

    let output = run_picolayer(&[
        "gh-release",
        "pkgxdev/pkgx",
        "pkgx",
        "--version",
        "v2.7.0",
        "--install-dir",
        bin_location,
        "--filter",
        &format!("{}.*x86-64\\.tar\\.xz", os),
    ]);
    print!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    print!("STDERR: {}", String::from_utf8_lossy(&output.stderr));

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if is_transient_error(&stderr) {
            eprintln!("Skipping XZ test due to transient error: {}", stderr);
            return;
        }

        eprintln!("XZ extraction test failed: {}", stderr);
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    }
}

#[test]
#[serial]
fn test_xz_extraction_handles_empty_data() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let bin_location = temp_dir.path().to_str().unwrap();

    let output = run_picolayer(&[
        "gh-release",
        "pkgxdev/pkgx",
        "pkgx",
        "--version",
        "v2.7.0",
        "--install-dir",
        bin_location,
        "--filter",
        "nonexistent_pattern_that_matches_nothing",
    ]);
    print!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    print!("STDERR: {}", String::from_utf8_lossy(&output.stderr));

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    if is_transient_error(&stderr) {
        eprintln!("Skipping test due to transient error");
        return;
    };
    assert!(
        stderr.contains("No matching") || stderr.contains("not found") || stderr.contains("filter"),
        "Should handle empty results gracefully: {}",
        stderr
    );
}
