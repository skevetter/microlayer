mod common;
#[cfg(target_os = "linux")]
use common::run_picolayer;
#[cfg(target_os = "linux")]
use serial_test::serial;

#[test]
#[serial]
#[cfg(target_os = "linux")]
fn test_apt_get_installation() {
    // Perform update before installing any package to ensure lists are populated
    std::process::Command::new("sudo")
        .arg("apt-get")
        .arg("update")
        .status()
        .expect("Failed to run apt-get update");

    // Expect /var/lib/apt/lists to exist
    let lists_path = std::path::Path::new("/var/lib/apt/lists");
    assert!(lists_path.exists(), "/var/lib/apt/lists does not exist");

    // Expect files in the lists directory
    let entries = std::fs::read_dir(lists_path).unwrap();
    assert!(entries.count() > 0, "/var/lib/apt/lists is empty");

    // Save the current state of apt lists
    let backup_path = "/tmp/apt_lists_backup";
    std::process::Command::new("sudo")
        .arg("cp")
        .arg("-r")
        .arg("/var/lib/apt/lists")
        .arg(backup_path)
        .status()
        .expect("Failed to backup apt lists");

    let output = run_picolayer(&["apt-get", "file"]);

    println!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    println!("STDERR: {}", String::from_utf8_lossy(&output.stderr));

    // Expect the command to succeed
    assert!(
        output.status.success(),
        "apt-get installation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Expect /var/lib/apt/lists to match the backup state
    let diff_output = std::process::Command::new("sudo")
        .arg("diff")
        .arg("-r")
        .arg("/var/lib/apt/lists")
        .arg(backup_path)
        .output()
        .expect("Failed to run diff on apt lists");

    assert!(
        diff_output.status.success(),
        "Apt lists differ from backup: {}",
        String::from_utf8_lossy(&diff_output.stdout)
    );

    // Clean up backup
    std::process::Command::new("sudo")
        .arg("rm")
        .arg("-rf")
        .arg(backup_path)
        .status()
        .expect("Failed to remove apt lists backup");
}

#[test]
#[serial]
#[cfg(target_os = "linux")]
fn test_apt_get_from_clean_state() {
    // Remove apt lists to simulate a clean state
    let rm_lists_output = std::process::Command::new("sudo")
        .arg("rm")
        .arg("-rf")
        .arg("/var/lib/apt/lists")
        .output()
        .expect("Failed to run sudo rm -rf /var/lib/apt/lists");

    println!(
        "Clean list successful: {}",
        rm_lists_output.status.success()
    );

    std::process::Command::new("sudo")
        .arg("mkdir")
        .arg("-p")
        .arg("/var/lib/apt/lists")
        .status()
        .expect("Failed to recreate /var/lib/apt/lists directory");

    // Install any package to trigger apt-get update
    let output = run_picolayer(&["apt-get", "curl"]);

    print!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    print!("STDERR: {}", String::from_utf8_lossy(&output.stderr));

    // Expect the command to succeed
    assert!(
        output.status.success(),
        "apt-get update failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Expect /var/lib/apt/lists to exist
    let lists_path = std::path::Path::new("/var/lib/apt/lists");
    assert!(
        lists_path.exists(),
        "/var/lib/apt/lists directory does not exist after apt-get update"
    );

    // Expect no files in the lists directory after apt-get update (e.g., restored cache to original state)
    let entries = std::fs::read_dir(lists_path).unwrap();
    assert!(
        entries.count() == 0,
        "/var/lib/apt/lists directory is not empty after apt-get update",
    );
}
