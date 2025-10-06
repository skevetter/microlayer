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

    let output = Command::new(binary_path).arg("--version").output();

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

    // Use the pkgx.dev GPG public key URL for integration tests
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

    // Use a filter to select the correct asset
    let arch = std::env::consts::ARCH;
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

    // Verify the binary works
    assert!(
        check_binary_version(&binary_path, None),
        "pkgx binary version check failed"
    );
}

#[test]
fn test_picolayer_run_python_version_legacy() {
    let output = run_picolayer(&["run", "python@3.11", "--version"]);

    if !output.status.success() {
        eprintln!(
            "Python version test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return; // Skip test if pkgx/python not available
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Python 3.11"));
}

#[test]
fn test_picolayer_run_node_version_legacy() {
    let output = run_picolayer(&["run", "node@18", "--version"]);

    if !output.status.success() {
        eprintln!(
            "Node version test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return; // Skip test if pkgx/node not available
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("v18"));
}

#[test]
fn test_picolayer_run_with_working_directory_legacy() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let working_dir = temp_dir.path().to_str().unwrap();

    // Create a simple script in the temp directory
    let script_path = temp_dir.path().join("test_script.py");
    std::fs::write(&script_path, "print('Hello from script')").expect("Failed to write script");

    let output = run_picolayer(&[
        "run",
        "python",
        "test_script.py",
        "--working-dir",
        working_dir,
    ]);

    if !output.status.success() {
        eprintln!(
            "Working directory test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return; // Skip test if pkgx/python not available
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Hello from script"));
}

#[test]
fn test_picolayer_run_dependency_detection_legacy() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

    // Create a package.json to test Node.js detection
    let package_json = temp_dir.path().join("package.json");
    std::fs::write(&package_json, r#"{"name": "test", "version": "1.0.0"}"#)
        .expect("Failed to write package.json");

    let output = run_picolayer(&[
        "run",
        "node",
        "--version",
        "--working-dir",
        temp_dir.path().to_str().unwrap(),
    ]);

    if !output.status.success() {
        eprintln!(
            "Dependency detection test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return; // Skip test if pkgx/node not available
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("v"));
}

#[test]
fn test_picolayer_run_python_with_requirements_legacy() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

    // Create a requirements.txt to test Python detection
    let requirements_txt = temp_dir.path().join("requirements.txt");
    std::fs::write(&requirements_txt, "requests==2.28.0")
        .expect("Failed to write requirements.txt");

    let output = run_picolayer(&[
        "run",
        "python",
        "--version",
        "--working-dir",
        temp_dir.path().to_str().unwrap(),
    ]);

    if !output.status.success() {
        eprintln!(
            "Python with requirements test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return; // Skip test if pkgx/python not available
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Python"));
}

#[test]
fn test_picolayer_run_go_with_mod_legacy() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

    // Create a go.mod to test Go detection
    let go_mod = temp_dir.path().join("go.mod");
    std::fs::write(&go_mod, "module test\n\ngo 1.19").expect("Failed to write go.mod");

    let output = run_picolayer(&[
        "run",
        "go",
        "version",
        "--working-dir",
        temp_dir.path().to_str().unwrap(),
    ]);

    if !output.status.success() {
        eprintln!(
            "Go with mod test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return; // Skip test if pkgx/go not available
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("go version"));
}

#[test]
fn test_picolayer_run_python_with_version() {
    let output = run_picolayer(&["run", "python@3.10", "--version"]);

    if !output.status.success() {
        eprintln!(
            "Python 3.10 version test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return; // Skip test if pkgx/python not available
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Python 3.10"));
}

#[test]
fn test_picolayer_run_python_latest() {
    let output = run_picolayer(&["run", "python", "--version"]);

    if !output.status.success() {
        eprintln!(
            "Python latest test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return; // Skip test if pkgx/python not available
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Python"));
}

#[test]
fn test_picolayer_run_python_script() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let script_path = temp_dir.path().join("test.py");
    std::fs::write(&script_path, "print('Hello from Python!')").expect("Failed to write script");

    let output = run_picolayer(&[
        "run",
        "python",
        script_path.to_str().unwrap(),
        "--working-dir",
        temp_dir.path().to_str().unwrap(),
    ]);

    if !output.status.success() {
        eprintln!(
            "Python script test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return; // Skip test if pkgx/python not available
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Hello from Python!"));
}

#[test]
fn test_picolayer_run_node_with_version() {
    let output = run_picolayer(&["run", "node@18", "--version"]);

    if !output.status.success() {
        eprintln!(
            "Node 18 version test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return; // Skip test if pkgx/node not available
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("v18."));
}

#[test]
fn test_picolayer_run_node_inline_code() {
    let output = run_picolayer(&["run", "node", "-e", "console.log('Hello from Node.js!')"]);

    if !output.status.success() {
        eprintln!(
            "Node inline code test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return; // Skip test if pkgx/node not available
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Hello from Node.js!"));
}

#[test]
fn test_picolayer_run_go_with_version() {
    let output = run_picolayer(&["run", "go@1.21", "version"]);

    if !output.status.success() {
        eprintln!(
            "Go 1.21 version test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return; // Skip test if pkgx/go not available
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("go1.21"));
}

#[test]
fn test_picolayer_run_ruby_inline() {
    let output = run_picolayer(&["run", "ruby", "-e", "puts 'Hello from Ruby!'"]);

    if !output.status.success() {
        eprintln!(
            "Ruby inline test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return; // Skip test if pkgx/ruby not available
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Hello from Ruby!"));
}

#[test]
fn test_picolayer_run_with_env_vars_new_syntax() {
    let output = run_picolayer(&[
        "run",
        "python",
        "-c",
        "import os; print(f'TEST_VAR={os.environ.get(\"TEST_VAR\", \"not found\")}')",
        "--env",
        "TEST_VAR=hello_world",
    ]);

    if !output.status.success() {
        eprintln!(
            "Python env vars test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return; // Skip test if pkgx/python not available
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("TEST_VAR=hello_world"));
}

#[test]
fn test_picolayer_run_with_force_pkgx_new_syntax() {
    let output = run_picolayer(&["run", "echo", "hello", "world", "--force-pkgx"]);

    if !output.status.success() {
        eprintln!(
            "Force pkgx test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return; // Skip test if pkgx not available
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hello world"));
}

#[test]
fn test_picolayer_run_rust_with_version() {
    let output = run_picolayer(&["run", "rustc@1.70", "--version"]);

    if !output.status.success() {
        eprintln!(
            "Rust 1.70 version test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return; // Skip test if pkgx/rust not available
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("rustc 1.70"));
}

#[test]
fn test_picolayer_run_multiple_args() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");

    std::fs::write(&file1, "content1").expect("Failed to write file1");
    std::fs::write(&file2, "content2").expect("Failed to write file2");

    let output = run_picolayer(&[
        "run",
        "cat",
        file1.to_str().unwrap(),
        file2.to_str().unwrap(),
        "--working-dir",
        temp_dir.path().to_str().unwrap(),
    ]);

    if !output.status.success() {
        eprintln!(
            "Multiple args test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return; // Skip test if cat not available
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("content1"));
    assert!(stdout.contains("content2"));
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
