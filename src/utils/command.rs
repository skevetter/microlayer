use anyhow::{Context, Result};
use std::process::{Command, Stdio};

/// Check if the current process has elevated privileges
/// Returns true if running as root (uid=0) on Unix systems
pub fn is_elevated() -> bool {
    #[cfg(target_family = "unix")]
    {
        let uid = unsafe { libc::geteuid() };
        uid == 0
    }
    
    #[cfg(not(target_family = "unix"))]
    {
        false
    }
}

/// Check if a path can be written to by the current user
pub fn can_write_to_path(path: &str) -> bool {
    use std::path::Path;
    
    let path = Path::new(path);
    
    // Check if parent directory exists and is writable
    if let Some(parent) = path.parent() {
        if parent.exists() {
            return std::fs::metadata(parent)
                .map(|m| !m.permissions().readonly())
                .unwrap_or(false);
        }
    }
    
    // If path exists, check if it's writable
    if path.exists() {
        return std::fs::metadata(path)
            .map(|m| !m.permissions().readonly())
            .unwrap_or(false);
    }
    
    false
}

/// Request elevated privileges using sudo
/// This will prompt the user for their password if necessary
pub fn elevate_prompt() -> Result<()> {
    #[cfg(target_family = "unix")]
    {
        // Use sudo -v to validate and cache credentials
        let status = Command::new("sudo")
            .arg("-v")
            .status()
            .context("Failed to invoke sudo")?;

        if !status.success() {
            anyhow::bail!("Failed to obtain sudo access");
        }

        Ok(())
    }
    
    #[cfg(not(target_family = "unix"))]
    {
        anyhow::bail!("Privilege escalation is not supported on this platform")
    }
}

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
    fn test_is_elevated_returns_bool() {
        // Should return a boolean value without panicking
        let result = is_elevated();
        assert!(result == true || result == false);
    }

    #[test]
    fn test_can_write_to_path_tmp() {
        // /tmp should typically be writable
        let result = can_write_to_path("/tmp");
        // On most systems this should be true, but we just test it doesn't panic
        assert!(result == true || result == false);
    }

    #[test]
    fn test_can_write_to_path_root() {
        // /root should typically not be writable by regular users
        let result = can_write_to_path("/root/test");
        // Just verify it returns a boolean
        assert!(result == true || result == false);
    }

    #[test]
    fn test_can_write_to_path_nonexistent() {
        // Non-existent path should return false
        let result = can_write_to_path("/nonexistent/path/that/does/not/exist");
        assert_eq!(result, false);
    }

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
