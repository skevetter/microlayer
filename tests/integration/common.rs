use std::path::Path;
use std::process::Command;

pub const PICOLAYER_BIN: &str = env!("CARGO_BIN_EXE_picolayer");

pub fn run_picolayer(args: &[&str]) -> std::process::Output {
    Command::new(PICOLAYER_BIN)
        .args(args)
        .output()
        .expect("Failed to execute picolayer")
}

pub fn is_transient_error(stderr: &str) -> bool {
    stderr.contains("403 Forbidden")
        || stderr.contains("rate limit")
        || stderr.contains("API rate limit")
        || stderr.contains("connection")
}

pub fn binary_exists(path: &str) -> bool {
    Path::new(path).exists()
}

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
