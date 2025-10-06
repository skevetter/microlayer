use crate::common::run_picolayer;
use std::process::Command;

#[test]
#[cfg(target_os = "linux")]
fn test_apt_get_installation() {
    let has_apt = Command::new("which")
        .arg("apt-get")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_apt {
        eprintln!("Skipping apt-get test: apt-get not available");
        return;
    }

    let output = run_picolayer(&["apt-get", "file"]);

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
fn test_apt_get_with_ppas() {
    let has_apt = Command::new("which")
        .arg("apt-get")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_apt {
        eprintln!("Skipping apt-get PPA test: apt-get not available");
        return;
    }

    let output = run_picolayer(&["apt-get", "file", "--ppas", "ppa:git-core/ppa"]);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("permission denied") || stderr.contains("root") {
            eprintln!("Skipping apt-get PPA test: requires root permissions");
            return;
        }
    }
}
