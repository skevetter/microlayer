use anyhow::{Context, Result};
use std::process::{Command, Stdio};

/// Execute a shell command and return success/failure
pub fn execute(cmd: &str) -> Result<()> {
    execute_with_output(cmd, true)?;
    Ok(())
}

/// Execute a shell command and optionally capture output
pub fn execute_with_output(cmd: &str, print_output: bool) -> Result<String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdout(if print_output { Stdio::inherit() } else { Stdio::piped() })
        .stderr(if print_output { Stdio::inherit() } else { Stdio::piped() })
        .output()
        .context(format!("Failed to execute command: {}", cmd))?;

    if !output.status.success() {
        anyhow::bail!("Command failed: {}", cmd);
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(stdout)
}

/// Execute a command and return exit code (don't fail)
pub fn execute_status(cmd: &str) -> Result<i32> {
    let status = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context(format!("Failed to execute command: {}", cmd))?;

    Ok(status.code().unwrap_or(-1))
}
