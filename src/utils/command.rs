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
        .stdout(if print_output {
            Stdio::inherit()
        } else {
            Stdio::piped()
        })
        .stderr(if print_output {
            Stdio::inherit()
        } else {
            Stdio::piped()
        })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_status_true() {
        let result = execute_status("true");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_execute_status_false() {
        let result = execute_status("false");
        assert!(result.is_ok());
        assert_ne!(result.unwrap(), 0);
    }

    #[test]
    fn test_execute_with_output_echo() {
        let result = execute_with_output("echo hello", false);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("hello"));
    }

    #[test]
    fn test_execute_success() {
        let result = execute("true");
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_failure() {
        let result = execute("false");
        assert!(result.is_err());
    }
}
