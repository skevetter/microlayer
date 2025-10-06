use crate::common::run_picolayer;

#[test]
fn test_main_help() {
    let output = run_picolayer(&["--help"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("picolayer"));
    assert!(stdout.contains("Ensures minimal container layers"));
}

#[test]
fn test_main_version() {
    let output = run_picolayer(&["--version"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("picolayer"));
}

#[test]
fn test_apt_get_help() {
    let output = run_picolayer(&["apt-get", "--help"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("apt-get"));
}

#[test]
fn test_apk_help() {
    let output = run_picolayer(&["apk", "--help"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("apk"));
}

#[test]
fn test_brew_help() {
    let output = run_picolayer(&["brew", "--help"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("brew"));
}

#[test]
fn test_gh_release_help() {
    let output = run_picolayer(&["gh-release", "--help"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("gh-release"));
}

#[test]
fn test_run_help() {
    let output = run_picolayer(&["run", "--help"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("run"));
}
