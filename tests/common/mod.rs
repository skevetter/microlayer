//! Common utilities for integration tests

use std::path::Path;
use std::process::Command;

/// Path to the picolayer binary for testing
#[allow(dead_code)]
pub const PICOLAYER_BIN: &str = env!("CARGO_BIN_EXE_picolayer");

/// Run picolayer with the given arguments and return the output
#[allow(dead_code)]
pub fn run_picolayer(args: &[&str]) -> std::process::Output {
    let output = Command::new(PICOLAYER_BIN)
        .args(args)
        .output()
        .expect("Failed to execute picolayer");
    
    // Print STDOUT and STDERR to help with debugging
    println!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    println!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
    
    output
}

/// Check if an error message indicates a transient error that should be retried or ignored
#[allow(dead_code)]
pub fn is_transient_error(stderr: &str) -> bool {
    stderr.contains("403 Forbidden")
        || stderr.contains("rate limit")
        || stderr.contains("API rate limit")
        || stderr.contains("connection")
}

/// Check if a binary exists at the given path
#[allow(dead_code)]
pub fn binary_exists(path: &str) -> bool {
    Path::new(path).exists()
}

/// Check if a binary exists and optionally verify it contains expected version info
#[allow(dead_code)]
pub fn check_binary_version(binary_path: &str, expected_substring: Option<&str>) -> bool {
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
