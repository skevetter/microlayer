mod common;
#[cfg(target_os = "linux")]
use common::run_picolayer;
#[cfg(target_os = "linux")]
use serial_test::serial;

#[test]
#[serial]
#[cfg(target_os = "linux")]
fn test_apt_get_installation() {
    let output = run_picolayer(&["apt-get", "file"]);

    assert!(
        output.status.success(),
        "apt-get installation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
