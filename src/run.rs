use anyhow::{Context, Result};
use std::path::Path;
use std::process::{Command, Stdio};

/// Execute a command using pkgx for dependency management
pub fn execute(
    command: &str,
    working_dir: &str,
    env_vars: &[String],
    _force_pkgx: bool,
) -> Result<()> {
    // Change to working directory
    let working_path = Path::new(working_dir);
    if !working_path.exists() {
        anyhow::bail!("Working directory does not exist: {}", working_dir);
    }

    println!("Executing: {}", command);
    println!("Working directory: {}", working_dir);

    // Parse environment variables
    let mut env_map = Vec::new();
    for env_var in env_vars {
        if let Some((key, value)) = env_var.split_once('=') {
            env_map.push((key.to_string(), value.to_string()));
        } else {
            anyhow::bail!(
                "Invalid environment variable format: {} (expected key=value)",
                env_var
            );
        }
    }

    #[cfg(feature = "pkgx-integration")]
    {
        if _force_pkgx || !check_pkgx_binary() {
            return execute_with_pkgx_library(command, working_path, &env_map);
        }
    }

    execute_with_pkgx_binary(command, working_path, &env_map)
}

#[cfg(feature = "pkgx-integration")]
fn execute_with_pkgx_library(
    command: &str,
    working_path: &Path,
    env_map: &[(String, String)],
) -> Result<()> {
    // TODO: Implement pkgx library integration
    // For now, fall back to binary
    println!("Note: pkgx library integration is under development, using binary");
    execute_with_pkgx_binary(command, working_path, env_map)
}

fn execute_with_pkgx_binary(
    command: &str,
    working_path: &Path,
    env_map: &[(String, String)],
) -> Result<()> {
    // Check if pkgx is available
    let pkgx_available = check_pkgx_binary();

    if !pkgx_available {
        anyhow::bail!(
            "pkgx is not available. Please install pkgx from https://pkgx.sh\n\
             Installation: curl -fsS https://pkgx.sh | sh"
        );
    }

    // Execute command with pkgx
    // pkgx will automatically detect and install required dependencies
    let mut cmd = Command::new("pkgx");

    // Add the command to run
    cmd.arg(command);

    // Set working directory
    cmd.current_dir(working_path);

    // Add environment variables
    for (key, value) in env_map {
        cmd.env(key, value);
    }

    // Inherit stdio so output is visible
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());
    cmd.stdin(Stdio::inherit());

    // Execute the command
    let status = cmd
        .status()
        .context("Failed to execute command with pkgx")?;

    if !status.success() {
        anyhow::bail!(
            "Command failed with exit code: {}",
            status.code().unwrap_or(-1)
        );
    }

    println!("Command executed successfully!");
    Ok(())
}

fn check_pkgx_binary() -> bool {
    Command::new("pkgx")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
