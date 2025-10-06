use crate::common::run_picolayer;
use std::process::Command;

#[test]
#[cfg(target_os = "linux")]
fn test_apk_installation() {
    let has_apk = Command::new("which")
        .arg("apk")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_apk {
        eprintln!("Skipping apk test: apk not available");
        return;
    }

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
#[cfg(target_os = "linux")]
fn test_apk_multiple_packages() {
    let has_apk = Command::new("which")
        .arg("apk")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_apk {
        eprintln!("Skipping apk multiple packages test: apk not available");
        return;
    }

    let output = run_picolayer(&["apk", "curl,git"]);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("permission denied") || stderr.contains("root") {
            eprintln!("Skipping apk multiple packages test: requires root permissions");
            return;
        }
    }
}
