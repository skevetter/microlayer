use crate::utils;
use anyhow::Result;
use log::{debug, info, trace, warn};
use std::os::unix::fs::PermissionsExt;
use std::process::{Command, ExitStatus};
use std::{fs, path::Path};
use thiserror::Error;

// ---------------------------------------------------
// Check Sudo
// source: https://gitlab.com/dns2utf8/sudo.rs/
// ---------------------------------------------------

/// Errors that can occur during privilege operations
#[derive(Error, Debug)]
pub enum PrivilegeError {
    #[error("Failed to execute sudo command")]
    SudoExecution,
    #[error("Failed to get current executable path")]
    ExecutablePath,
    #[error("Privilege escalation was denied or failed")]
    EscalationFailed,
    #[error("System call failed: {0}")]
    SystemCall(String),
    #[error("Root privileges are required but not available")]
    #[allow(dead_code)]
    RootRequired,
    #[error("Sudo is not available on this system")]
    SudoNotAvailable,
}

/// Cross-platform representation of the current process privilege state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivilegeLevel {
    /// Root (Linux/macOS/Unix) or Administrator (Windows)
    Root,
    /// Running as a normal user
    User,
    /// Started from SUID - can escalate to root without password
    Suid,
}

/// Configuration for privilege escalation
#[derive(Debug, Default)]
pub struct EscalationConfig<'a> {
    /// Environment variable prefixes to preserve during escalation
    pub preserve_env_prefixes: &'a [&'a str],
    /// Whether to preserve RUST_BACKTRACE (default: true)
    pub preserve_backtrace: bool,
    /// Custom sudo binary path (default: /usr/bin/sudo)
    pub sudo_path: Option<&'a str>,
}

impl<'a> EscalationConfig<'a> {
    /// Create a new escalation configuration
    pub fn new() -> Self {
        Self {
            preserve_backtrace: true,
            ..Default::default()
        }
    }

    #[allow(dead_code)]
    /// Set environment prefixes to preserve
    pub fn with_env_prefixes(mut self, prefixes: &'a [&'a str]) -> Self {
        self.preserve_env_prefixes = prefixes;
        self
    }

    #[allow(dead_code)]
    /// Set whether to preserve RUST_BACKTRACE
    pub fn preserve_backtrace(mut self, preserve: bool) -> Self {
        self.preserve_backtrace = preserve;
        self
    }

    #[allow(dead_code)]
    /// Set custom sudo binary path
    pub fn with_sudo_path(mut self, path: &'a str) -> Self {
        self.sudo_path = Some(path);
        self
    }
}

const DEFAULT_SUDO_PATH: &str = "/usr/bin/sudo";

#[cfg(unix)]
/// Check the current privilege level of the process
///
/// This function examines the real UID (getuid) and effective UID (geteuid)
/// to determine the privilege state:
/// - Both 0: Running as root
/// - Real UID != 0, Effective UID = 0: SUID binary
/// - Both != 0: Regular user
pub fn check_privilege_level() -> PrivilegeLevel {
    let uid = unsafe { libc::getuid() };
    let euid = unsafe { libc::geteuid() };

    trace!("Process UIDs - real: {}, effective: {}", uid, euid);

    match (uid, euid) {
        (0, 0) => PrivilegeLevel::Root,
        (_, 0) => PrivilegeLevel::Suid,
        (_, _) => PrivilegeLevel::User,
    }
}

#[cfg(unix)]
/// Ensure the process has sudo/root privileges
///
/// This function will:
/// 1. Check if already running as root - if yes, return Ok(())
/// 2. If SUID, activate SUID privileges
/// 3. If regular user, restart the process with sudo
/// 4. Return an error if escalation is not possible
///
/// Unlike `escalate_if_needed()`, this function is designed to be called
/// at specific points where root privileges are absolutely required.
///
/// # Errors
///
/// Returns `PrivilegeError::RootRequired` if the process cannot obtain root privileges,
/// or other errors if the escalation process fails.
pub fn ensure_sudo() -> Result<()> {
    match check_privilege_level() {
        PrivilegeLevel::Root => {
            trace!("Already running as root");
            Ok(())
        }
        PrivilegeLevel::Suid => {
            debug!("Activating SUID privileges to ensure root access");
            activate_suid_privileges()
        }
        PrivilegeLevel::User => {
            debug!("Attempting privilege escalation");
            escalate_with_config(&EscalationConfig::new())?;
            Ok(())
        }
    }
}

#[cfg(unix)]
#[allow(dead_code)]
/// Ensure sudo privileges with custom configuration
///
/// Similar to `ensure_sudo()` but allows custom escalation configuration.
///
/// # Arguments
///
/// * `config` - Configuration for the escalation process
pub fn ensure_sudo_with_config(config: &EscalationConfig) -> Result<()> {
    match check_privilege_level() {
        PrivilegeLevel::Root => {
            trace!("Already running as root");
            Ok(())
        }
        PrivilegeLevel::Suid => {
            debug!("Activating SUID privileges to ensure root access");
            activate_suid_privileges()
        }
        PrivilegeLevel::User => {
            debug!("Root privileges required, attempting escalation with custom config");
            escalate_with_config(config)?;
            Ok(())
        }
    }
}

#[cfg(unix)]
/// Check if sudo is available on the system
///
/// This function checks if the sudo binary exists and is executable.
/// It doesn't check if the current user can actually use sudo.
///
/// # Returns
///
/// `true` if sudo binary is found and executable, `false` otherwise.
pub fn is_sudo_available() -> bool {
    is_sudo_available_at_path(DEFAULT_SUDO_PATH)
}

#[cfg(unix)]
/// Check if sudo is available at a specific path
///
/// # Arguments
///
/// * `sudo_path` - Path to the sudo binary to check
pub fn is_sudo_available_at_path(sudo_path: &str) -> bool {
    Path::new(sudo_path).is_file()
        && std::fs::metadata(sudo_path)
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
}

#[cfg(unix)]
/// Check if the current user can use sudo (without actually running it)
///
/// This function runs `sudo -n true` to check if the user can execute
/// sudo without a password prompt. This is useful for checking sudo
/// availability before attempting operations that require it.
///
/// # Returns
///
/// `true` if the user can use sudo, `false` otherwise.
pub fn can_use_sudo() -> bool {
    can_use_sudo_at_path(DEFAULT_SUDO_PATH)
}

#[cfg(unix)]
/// Check if the current user can use sudo at a specific path
///
/// # Arguments
///
/// * `sudo_path` - Path to the sudo binary to test
pub fn can_use_sudo_at_path(sudo_path: &str) -> bool {
    if !is_sudo_available_at_path(sudo_path) {
        return false;
    }

    Command::new(sudo_path)
        .args(["-n", "true"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(unix)]
#[allow(dead_code)]
/// Ensure root privileges are available, but don't escalate yet
///
/// This function checks if root privileges can be obtained without
/// actually escalating. It's useful for early validation.
///
/// # Errors
///
/// Returns an error if root privileges cannot be obtained.
pub fn require_sudo_available() -> Result<()> {
    match check_privilege_level() {
        PrivilegeLevel::Root | PrivilegeLevel::Suid => {
            trace!("Root privileges already available");
            Ok(())
        }
        PrivilegeLevel::User => {
            if !is_sudo_available() {
                return Err(PrivilegeError::SudoNotAvailable.into());
            }

            if !can_use_sudo() {
                return Err(PrivilegeError::RootRequired.into());
            }

            debug!("Sudo is available and user can use it");
            Ok(())
        }
    }
}

#[cfg(unix)]
#[allow(dead_code)]
/// Validate the complete sudo setup
///
/// This function performs comprehensive validation of the sudo setup,
/// using all the different PrivilegeError variants appropriately.
///
/// # Errors
///
/// Returns specific PrivilegeError variants based on what fails.
pub fn validate_sudo_setup() -> Result<()> {
    // Check if we can get executable path
    let _args = get_process_args().map_err(|_| PrivilegeError::ExecutablePath)?;

    // Check if sudo is available
    if !is_sudo_available() {
        return Err(PrivilegeError::SudoNotAvailable.into());
    }

    // Check if user can actually use sudo
    if !can_use_sudo() {
        return Err(PrivilegeError::RootRequired.into());
    }

    // Try a simple sudo command to verify it works
    let output = Command::new(DEFAULT_SUDO_PATH)
        .args(["-n", "true"])
        .output()
        .map_err(|_| PrivilegeError::SudoExecution)?;

    if !output.status.success() {
        return Err(PrivilegeError::EscalationFailed.into());
    }

    debug!("Sudo setup validation completed successfully");
    Ok(())
}

#[cfg(unix)]
#[allow(dead_code)]
/// Escalate privileges if needed using default configuration
///
/// This is a convenience function that calls `escalate_with_config` with
/// default settings (preserve RUST_BACKTRACE only).
///
/// # Errors
///
/// Returns an error if privilege escalation fails or if the process
/// cannot be restarted with sudo.
pub fn escalate_if_needed() -> Result<PrivilegeLevel> {
    escalate_with_config(&EscalationConfig::new())
}

#[cfg(unix)]
/// Escalate privileges with custom environment preservation
///
/// This function will:
/// 1. Check current privilege level
/// 2. If already root, return immediately
/// 3. If SUID, activate the SUID privileges
/// 4. If user, restart the process with sudo
///
/// # Arguments
///
/// * `config` - Configuration for the escalation process
///
/// # Errors
///
/// Returns an error if privilege escalation fails, sudo is not available,
/// or the process cannot be restarted.
pub fn escalate_with_config(config: &EscalationConfig) -> Result<PrivilegeLevel> {
    let current_level = check_privilege_level();
    trace!("Current privilege level: {:?}", current_level);

    match current_level {
        PrivilegeLevel::Root => {
            debug!("Already running as root");
            Ok(current_level)
        }
        PrivilegeLevel::Suid => {
            debug!("Activating SUID privileges");
            activate_suid_privileges()?;
            Ok(current_level)
        }
        PrivilegeLevel::User => {
            debug!("Escalating privileges via sudo");

            if !is_sudo_available() {
                return Err(PrivilegeError::SudoNotAvailable.into());
            }

            if !can_use_sudo() {
                return Err(PrivilegeError::EscalationFailed.into());
            }

            restart_with_sudo(config)?;
            unreachable!("restart_with_sudo should not return")
        }
    }
}

#[cfg(unix)]
/// Activate SUID privileges by setting the real UID to the effective UID
///
/// # Errors
///
/// Returns an error if the setuid system call fails.
fn activate_suid_privileges() -> Result<()> {
    let result = unsafe { libc::setuid(0) };

    if result == 0 {
        trace!("Successfully activated SUID privileges");
        Ok(())
    } else {
        Err(PrivilegeError::SystemCall("setuid failed".to_string()).into())
    }
}

#[cfg(unix)]
/// Restart the current process with sudo, preserving specified environment variables
fn restart_with_sudo(config: &EscalationConfig) -> Result<()> {
    info!("Restarting process with sudo for privilege escalation");
    let args = get_process_args()?;
    let sudo_path = config.sudo_path.unwrap_or(DEFAULT_SUDO_PATH);

    if !is_sudo_available_at_path(sudo_path) {
        return Err(PrivilegeError::SudoNotAvailable.into());
    }

    let mut command = Command::new(sudo_path);

    add_environment_variables(&mut command, config)?;
    command.args(args);
    debug!("Executing {:?}", command);

    let mut child = command.spawn().map_err(|e| {
        debug!("Failed to spawn sudo process: {}", e);
        PrivilegeError::SudoExecution
    })?;

    let exit_status = child.wait().map_err(|e| {
        debug!("Failed to wait for sudo process: {}", e);
        PrivilegeError::SudoExecution
    })?;

    if !exit_status.success() {
        let exit_code = exit_status.code().unwrap_or(-1);
        debug!("Sudo process failed with exit code: {}", exit_code);
        return Err(PrivilegeError::EscalationFailed.into());
    }

    std::process::exit(get_exit_code(exit_status));
}

/// Get the current process arguments, replacing argv[0] with the absolute path if possible
fn get_process_args() -> Result<Vec<String>> {
    let mut args: Vec<String> = std::env::args().collect();

    if args.is_empty() {
        return Err(PrivilegeError::ExecutablePath.into());
    }

    match std::env::current_exe() {
        Ok(current_exe) => {
            if let Some(exe_str) = current_exe.to_str() {
                args[0] = exe_str.to_string();
                trace!("Using absolute path: {}", exe_str);
            } else {
                warn!("Current executable path contains invalid UTF-8, using original arg[0]");
            }
        }
        Err(e) => {
            warn!("Failed to get current executable path: {}", e);
            if args[0].is_empty() || !args[0].starts_with('/') {
                debug!("Executable path resolution failed and args[0] is relative");
                return Err(PrivilegeError::ExecutablePath.into());
            }
        }
    }

    Ok(args)
}

/// Add environment variables to the sudo command based on configuration
fn add_environment_variables(command: &mut Command, config: &EscalationConfig) -> Result<()> {
    let logging_vars = ["RUST_LOG", "RUST_LOG_STYLE"];

    for var_name in &logging_vars {
        if let Ok(value) = std::env::var(var_name) {
            trace!("Preserving logging variable: {}={}", var_name, value);
            command.arg(format!("{}={}", var_name, value));
        }
    }

    if config.preserve_backtrace
        && let Ok(backtrace_value) = std::env::var("RUST_BACKTRACE")
    {
        let normalized_value = normalize_backtrace_value(&backtrace_value);
        if let Some(value) = normalized_value {
            trace!("Preserving RUST_BACKTRACE={}", value);
            command.arg(format!("RUST_BACKTRACE={}", value));
        }
    }

    if !config.preserve_env_prefixes.is_empty() {
        for (name, value) in std::env::vars() {
            if name == "RUST_BACKTRACE" {
                continue; // Already handled above
            }

            if config
                .preserve_env_prefixes
                .iter()
                .any(|prefix| name.starts_with(prefix))
            {
                trace!("Preserving environment variable: {}={}", name, value);
                command.arg(format!("{}={}", name, value));
            }
        }
    }

    Ok(())
}

/// Normalize RUST_BACKTRACE value to valid options
fn normalize_backtrace_value(value: &str) -> Option<&'static str> {
    match value.to_lowercase().as_str() {
        "" => None,
        "1" | "true" => Some("1"),
        "full" => Some("full"),
        invalid => {
            warn!(
                "Invalid RUST_BACKTRACE value '{:?}', defaulting to 'full'",
                invalid
            );
            Some("full")
        }
    }
}

/// Extract exit code from ExitStatus, defaulting to 1 if not available
fn get_exit_code(status: ExitStatus) -> i32 {
    status.code().unwrap_or(1)
}

#[allow(dead_code)]
/// Check if an IO error indicates permission issues
pub fn is_permission_error(error: &std::io::Error) -> bool {
    use std::io::ErrorKind;
    matches!(error.kind(), ErrorKind::PermissionDenied)
}

#[cfg(test)]
mod sudo_tests {
    use super::*;

    #[test]
    fn test_privilege_level_equality() {
        assert_eq!(PrivilegeLevel::Root, PrivilegeLevel::Root);
        assert_eq!(PrivilegeLevel::User, PrivilegeLevel::User);
        assert_eq!(PrivilegeLevel::Suid, PrivilegeLevel::Suid);

        assert_ne!(PrivilegeLevel::Root, PrivilegeLevel::User);
    }

    #[test]
    fn test_escalation_config_builder() {
        let config = EscalationConfig::new()
            .with_env_prefixes(&["TEST_", "APP_"])
            .preserve_backtrace(false)
            .with_sudo_path("/custom/sudo");

        assert_eq!(config.preserve_env_prefixes, &["TEST_", "APP_"]);
        assert!(!config.preserve_backtrace);
        assert_eq!(config.sudo_path, Some("/custom/sudo"));
    }

    #[test]
    fn test_normalize_backtrace_value() {
        assert_eq!(normalize_backtrace_value("1"), Some("1"));
        assert_eq!(normalize_backtrace_value("true"), Some("1"));
        assert_eq!(normalize_backtrace_value("full"), Some("full"));
        assert_eq!(normalize_backtrace_value(""), None);
        assert_eq!(normalize_backtrace_value("invalid"), Some("full"));
    }

    #[test]
    fn test_privilege_errors() {
        // Test that our error types work correctly
        let error = PrivilegeError::SudoNotAvailable;
        assert_eq!(error.to_string(), "Sudo is not available on this system");

        let error = PrivilegeError::SystemCall("test failed".to_string());
        assert_eq!(error.to_string(), "System call failed: test failed");

        let error = PrivilegeError::EscalationFailed;
        assert_eq!(
            error.to_string(),
            "Privilege escalation was denied or failed"
        );

        let error = PrivilegeError::SudoExecution;
        assert_eq!(error.to_string(), "Failed to execute sudo command");

        let error = PrivilegeError::ExecutablePath;
        assert_eq!(error.to_string(), "Failed to get current executable path");

        let error = PrivilegeError::RootRequired;
        assert_eq!(
            error.to_string(),
            "Root privileges are required but not available"
        );
    }

    #[cfg(unix)]
    #[test]
    fn test_check_privilege_level() {
        let level = check_privilege_level();
        assert!(matches!(
            level,
            PrivilegeLevel::Root | PrivilegeLevel::User | PrivilegeLevel::Suid
        ));
    }

    #[test]
    fn test_get_process_args() {
        let args = get_process_args().expect("Failed to get process args");
        assert!(!args.is_empty());
        assert!(!args[0].is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn test_is_sudo_available() {
        let available = is_sudo_available();
        println!("Sudo available: {}", available);
    }

    #[cfg(unix)]
    #[test]
    fn test_sudo_availability_functions() {
        let _ = is_sudo_available();
        let _ = can_use_sudo();
        let _ = require_sudo_available();
    }

    #[cfg(unix)]
    #[test]
    fn test_validate_sudo_setup() {
        match validate_sudo_setup() {
            Ok(_) => println!("Sudo setup is valid"),
            Err(e) => {
                println!("Sudo setup error: {}", e);
                let error_chain = e.chain().collect::<Vec<_>>();
                if let Some(privilege_error) = error_chain
                    .iter()
                    .find_map(|e| e.downcast_ref::<PrivilegeError>())
                {
                    match privilege_error {
                        PrivilegeError::SudoNotAvailable => println!("Sudo binary not found"),
                        PrivilegeError::RootRequired => println!("User cannot use sudo"),
                        PrivilegeError::SudoExecution => println!("Sudo execution failed"),
                        PrivilegeError::EscalationFailed => println!("Privilege escalation failed"),
                        PrivilegeError::ExecutablePath => {
                            println!("Cannot determine executable path")
                        }
                        PrivilegeError::SystemCall(msg) => println!("System call failed: {}", msg),
                    }
                }
            }
        }
    }

    #[test]
    fn test_is_permission_error() {
        use std::io::{Error, ErrorKind};

        let perm_error = Error::new(ErrorKind::PermissionDenied, "Permission denied");
        assert!(is_permission_error(&perm_error));

        let other_error = Error::new(ErrorKind::NotFound, "File not found");
        assert!(!is_permission_error(&other_error));
    }
}

// ---------------------------------------------------
// Check Linux Distro
// ---------------------------------------------------

#[derive(Debug, PartialEq)]
pub enum LinuxDistro {
    Ubuntu,
    Debian,
    Alpine,
    Other,
}

/// Detect the Linux distribution
pub fn detect_distro() -> Result<LinuxDistro> {
    if let Ok(contents) = fs::read_to_string("/etc/os-release") {
        let mut kv = std::collections::HashMap::new();

        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(pos) = line.find('=') {
                let key = line[..pos].trim().to_ascii_uppercase();
                let mut val = line[pos + 1..]
                    .trim()
                    .trim_matches(|c| c == '"' || c == '\'')
                    .to_string();
                if key == "ID_LIKE" {
                    val = val.replace(&[',', ';'][..], " ");
                }
                kv.insert(key, val);
            }
        }

        let id = kv.get("ID").map(|s| s.as_str()).unwrap_or_default();
        let id_like = kv.get("ID_LIKE").map(|s| s.as_str()).unwrap_or_default();

        let matches_any = |target: &str| {
            if id.eq_ignore_ascii_case(target) {
                return true;
            }
            id_like
                .split_whitespace()
                .any(|token| token.eq_ignore_ascii_case(target))
        };

        if matches_any("ubuntu") {
            return Ok(LinuxDistro::Ubuntu);
        }
        if matches_any("alpine") {
            return Ok(LinuxDistro::Alpine);
        }
        if matches_any("debian") {
            return Ok(LinuxDistro::Debian);
        }
    }

    if fs::metadata("/etc/alpine-release").is_ok() {
        return Ok(LinuxDistro::Alpine);
    }
    if fs::metadata("/etc/debian_version").is_ok() {
        return Ok(LinuxDistro::Debian);
    }
    if let Ok(contents) = fs::read_to_string("/etc/lsb-release") {
        for line in contents.lines() {
            let line = line.trim();
            if let Some(pos) = line.find('=') {
                let key = &line[..pos];
                let val = line[pos + 1..]
                    .trim()
                    .trim_matches(|c| c == '"' || c == '\'');
                if key == "DISTRIB_ID" && val.eq_ignore_ascii_case("ubuntu") {
                    return Ok(LinuxDistro::Ubuntu);
                }
            }
        }
    }

    Ok(LinuxDistro::Other)
}

/// Check if the system is Ubuntu
pub fn is_ubuntu() -> bool {
    matches!(detect_distro(), Ok(LinuxDistro::Ubuntu))
}

/// Check if the system is Debian-like
pub fn is_debian_like() -> bool {
    matches!(
        detect_distro(),
        Ok(LinuxDistro::Ubuntu) | Ok(LinuxDistro::Debian)
    )
}

/// Check if the system is Alpine
pub fn is_alpine() -> bool {
    matches!(detect_distro(), Ok(LinuxDistro::Alpine))
}

#[cfg(test)]
mod distro_tests {
    use super::*;

    #[test]
    fn test_detect_distro_returns_result() {
        let result = detect_distro();
        assert!(result.is_ok());
    }
}

// ---------------------------------------------------
// Copy Files
// ---------------------------------------------------

#[allow(dead_code)]
pub fn copy_files(src: &Path, dest: &Path) -> Result<(), Box<dyn std::error::Error>> {
    utils::filesystem::atomic_copy_dir_preserve_symlinks(src, dest)
}

#[cfg(test)]
mod copy_tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_copy_files() {
        env_logger::builder().is_test(true).try_init().ok();

        let src_dir = tempdir().unwrap();
        let dest_dir = tempdir().unwrap();

        // Create a test file in the source directory
        let test_file_path = src_dir.path().join("test.txt");
        fs::write(&test_file_path, "Hello, world!").unwrap();

        // Copy files from the source to the destination directory
        let result = copy_files(src_dir.path(), dest_dir.path());
        assert!(result.is_ok());

        // Verify that the file was copied
        let copied_file_path = dest_dir.path().join("test.txt");
        assert!(fs::metadata(copied_file_path).is_ok());
    }
}
