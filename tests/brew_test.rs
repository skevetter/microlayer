mod common;

use common::run_picolayer;

#[cfg(target_os = "macos")]
use std::process::Command;

#[test]
#[cfg(target_os = "macos")]
fn test_brew_installation() {
    let has_brew = Command::new("which")
        .arg("brew")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_brew {
        eprintln!("Skipping brew test: Homebrew not available");
        return;
    }

    let output = run_picolayer(&["brew", "jq"]);

    if !output.status.success() {
        eprintln!("Brew output: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("Brew error: {}", String::from_utf8_lossy(&output.stderr));
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("Homebrew is not available"),
        "Brew command failed to detect Homebrew"
    );
}

#[test]
#[cfg(target_os = "macos")]
fn test_brew_multiple_packages() {
    let has_brew = Command::new("which")
        .arg("brew")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_brew {
        eprintln!("Skipping brew multiple packages test: Homebrew not available");
        return;
    }

    let output = run_picolayer(&["brew", "jq,tree"]);

    if !output.status.success() {
        eprintln!("Brew output: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("Brew error: {}", String::from_utf8_lossy(&output.stderr));
    }
}
