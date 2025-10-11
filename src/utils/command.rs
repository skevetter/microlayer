use anyhow::{Context, Result};
use std::process::{Command as StdCommand, Stdio};

pub struct CommandOutput {
    #[allow(dead_code)]
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

impl CommandOutput {
    pub fn is_success(&self) -> bool {
        self.exit_code == 0
    }
}

#[derive(Debug, Clone)]
pub enum ExecutionMode {
    Normal,
    Privileged,
    StatusOnly,
}

#[derive(Debug, Clone)]
pub enum OutputMode {
    Capture,
    Inherit,
    #[allow(dead_code)]
    Silent,
}

pub struct ElevationController;

impl ElevationController {
    /// Check if the current process is running with elevated privileges
    pub fn is_elevated() -> bool {
        #[cfg(unix)]
        {
            unsafe { libc::geteuid() == 0 }
        }
    }

    /// Get the elevation command for the current platform
    pub fn get_elevation_command() -> Option<(&'static str, Vec<&'static str>)> {
        if Self::is_elevated() {
            return None; // Already elevated
        }

        #[cfg(unix)]
        {
            if Self::command_exists("sudo") {
                Some(("sudo", vec![]))
            } else if Self::command_exists("su") {
                Some(("su", vec!["-c"]))
            } else if Self::command_exists("doas") {
                Some(("doas", vec![]))
            } else {
                None
            }
        }
    }

    /// Check if a command exists in PATH
    fn command_exists(cmd: &str) -> bool {
        StdCommand::new("which")
            .arg(cmd)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    /// Execute a command with elevation
    pub fn execute_elevated(mut command: StdCommand) -> Result<std::process::Output> {
        if Self::is_elevated() {
            return command
                .output()
                .context("Failed to execute already elevated command");
        }

        if let Some((elevation_cmd, elevation_args)) = Self::get_elevation_command() {
            let mut elevated_command = StdCommand::new(elevation_cmd);

            for arg in elevation_args {
                elevated_command.arg(arg);
            }

            let program = command.get_program().to_string_lossy().to_string();
            elevated_command.arg(&program);

            for arg in command.get_args() {
                elevated_command.arg(arg);
            }

            for (key, value) in command.get_envs() {
                if let Some(value) = value {
                    elevated_command.env(key, value);
                }
            }

            if let Some(current_dir) = command.get_current_dir() {
                elevated_command.current_dir(current_dir);
            }

            elevated_command
                .output()
                .context("Failed to execute elevated command")
        } else {
            anyhow::bail!("No elevation method available (sudo, doas, or su not found)")
        }
    }
}

pub struct CommandBuilder<'a> {
    command: &'a str,
    args: Vec<&'a str>,
    execution_mode: ExecutionMode,
    output_mode: OutputMode,
    fail_on_error: bool,
    working_directory: Option<&'a str>,
    environment: Vec<(&'a str, &'a str)>,
}

impl<'a> CommandBuilder<'a> {
    /// Create a new command builder with a command string
    pub fn new(cmd: &'a str) -> Self {
        Self {
            command: cmd,
            args: Vec::new(),
            execution_mode: ExecutionMode::Normal,
            output_mode: OutputMode::Capture,
            fail_on_error: true,
            working_directory: None,
            environment: Vec::new(),
        }
    }

    /// Create a new command builder with command and arguments
    pub fn with_args(cmd: &'a str, args: &[&'a str]) -> Self {
        Self {
            command: cmd,
            args: args.to_vec(),
            execution_mode: ExecutionMode::Normal,
            output_mode: OutputMode::Capture,
            fail_on_error: true,
            working_directory: None,
            environment: Vec::new(),
        }
    }

    /// Set execution mode (normal, privileged, or status-only)
    pub fn execution_mode(mut self, mode: ExecutionMode) -> Self {
        self.execution_mode = mode;
        self
    }

    /// Set output mode (capture, inherit, or silent)
    pub fn output_mode(mut self, mode: OutputMode) -> Self {
        self.output_mode = mode;
        self
    }

    /// Whether to fail on non-zero exit codes (default: true)
    pub fn fail_on_error(mut self, fail: bool) -> Self {
        self.fail_on_error = fail;
        self
    }

    /// Set working directory for the command
    pub fn working_directory(mut self, dir: &'a str) -> Self {
        self.working_directory = Some(dir);
        self
    }

    /// Add an environment variable
    #[allow(dead_code)]
    pub fn env(mut self, key: &'a str, value: &'a str) -> Self {
        self.environment.push((key, value));
        self
    }

    /// Add multiple environment variables
    pub fn envs(mut self, env_vars: &[(&'a str, &'a str)]) -> Self {
        self.environment.extend_from_slice(env_vars);
        self
    }

    /// Add an argument to the command
    pub fn arg(mut self, arg: &'a str) -> Self {
        self.args.push(arg);
        self
    }

    /// Add multiple arguments to the command
    pub fn args(mut self, args: &[&'a str]) -> Self {
        self.args.extend_from_slice(args);
        self
    }

    /// Execute
    pub fn execute(self) -> Result<CommandOutput> {
        self.execution_mode(ExecutionMode::Normal).run()
    }

    /// Execute with elevated privileges
    pub fn execute_privileged(self) -> Result<CommandOutput> {
        self.execution_mode(ExecutionMode::Privileged).run()
    }

    /// Execute and return status without failing on error
    pub fn execute_status(self) -> Result<CommandOutput> {
        self.execution_mode(ExecutionMode::StatusOnly)
            .fail_on_error(false)
            .run()
    }

    /// Execute the command with the configured settings
    pub fn run(self) -> Result<CommandOutput> {
        match self.execution_mode {
            ExecutionMode::Privileged => self.run_privileged(),
            ExecutionMode::Normal | ExecutionMode::StatusOnly => self.run_normal(),
        }
    }

    fn run_normal(self) -> Result<CommandOutput> {
        let command_name = self.get_command_description();
        let output = self
            .build_std_command()
            .output()
            .context(format!("Failed to execute command: {}", command_name))?;

        let command_output = CommandOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
        };

        if self.fail_on_error && !output.status.success() {
            anyhow::bail!(
                "Command failed: {} (exit code: {})\nstderr: {}",
                command_name,
                command_output.exit_code,
                command_output.stderr
            );
        }

        Ok(command_output)
    }

    fn run_privileged(self) -> Result<CommandOutput> {
        if ElevationController::is_elevated() {
            // Already elevated, run normally
            return self.execution_mode(ExecutionMode::Normal).run();
        }

        let command_name = self.get_command_description();
        let std_command = self.build_std_command();
        let output = ElevationController::execute_elevated(std_command).context(format!(
            "Failed to execute privileged command: {}",
            command_name
        ))?;

        let command_output = CommandOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
        };

        if self.fail_on_error && !output.status.success() {
            anyhow::bail!(
                "Privileged command failed: {} (exit code: {})\nstderr: {}",
                command_name,
                command_output.exit_code,
                command_output.stderr
            );
        }

        Ok(command_output)
    }

    fn get_command_description(&self) -> String {
        if self.args.is_empty() {
            self.command.to_string()
        } else {
            format!("{} {}", self.command, self.args.join(" "))
        }
    }

    fn build_std_command(&self) -> StdCommand {
        let mut command = StdCommand::new(self.command);

        // Add arguments
        for arg in &self.args {
            command.arg(arg);
        }

        // Set working directory if specified
        if let Some(dir) = self.working_directory {
            command.current_dir(dir);
        }

        // Set environment variables
        for (key, value) in &self.environment {
            command.env(key, value);
        }

        // Configure stdio based on output mode
        match self.output_mode {
            OutputMode::Capture => {
                command.stdout(Stdio::piped()).stderr(Stdio::piped());
            }
            OutputMode::Inherit => {
                command.stdout(Stdio::inherit()).stderr(Stdio::inherit());
            }
            OutputMode::Silent => {
                command.stdout(Stdio::null()).stderr(Stdio::null());
            }
        }

        command
    }
}

/// Command executor that holds references to external configuration
pub struct CommandExecutor<'a> {
    default_working_dir: Option<&'a str>,
    default_env: &'a [(&'a str, &'a str)],
    default_execution_mode: ExecutionMode,
    default_output_mode: OutputMode,
}

impl<'a> CommandExecutor<'a> {
    pub fn new() -> Self {
        Self {
            default_working_dir: None,
            default_env: &[],
            default_execution_mode: ExecutionMode::Normal,
            default_output_mode: OutputMode::Capture,
        }
    }

    #[allow(dead_code)]
    pub fn with_defaults(
        working_dir: Option<&'a str>,
        env: &'a [(&'a str, &'a str)],
        execution_mode: ExecutionMode,
        output_mode: OutputMode,
    ) -> Self {
        Self {
            default_working_dir: working_dir,
            default_env: env,
            default_execution_mode: execution_mode,
            default_output_mode: output_mode,
        }
    }

    /// Create a command builder with executor defaults
    pub fn command(&self, cmd: &'a str) -> CommandBuilder<'a> {
        let mut builder = CommandBuilder::new(cmd)
            .execution_mode(self.default_execution_mode.clone())
            .output_mode(self.default_output_mode.clone());

        if let Some(dir) = self.default_working_dir {
            builder = builder.working_directory(dir);
        }

        builder = builder.envs(self.default_env);

        builder
    }

    /// Create a command builder with args and executor defaults
    #[allow(dead_code)]
    pub fn command_with_args(&self, cmd: &'a str, args: &[&'a str]) -> CommandBuilder<'a> {
        let mut builder = CommandBuilder::with_args(cmd, args)
            .execution_mode(self.default_execution_mode.clone())
            .output_mode(self.default_output_mode.clone());

        if let Some(dir) = self.default_working_dir {
            builder = builder.working_directory(dir);
        }

        builder = builder.envs(self.default_env);

        builder
    }

    /// Check if elevation is available on this system
    #[allow(dead_code)]
    pub fn elevation_available(&self) -> bool {
        ElevationController::get_elevation_command().is_some() || ElevationController::is_elevated()
    }

    /// Check if currently running with elevated privileges
    pub fn is_elevated(&self) -> bool {
        ElevationController::is_elevated()
    }
}

impl<'a> Default for CommandExecutor<'a> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elevation_manager_is_elevated() {
        let is_elevated = ElevationController::is_elevated();
        println!("Is elevated: {}", is_elevated);
        assert!(is_elevated == true || is_elevated == false);
    }

    #[test]
    fn test_elevation_manager_command_exists() {
        let ls_exists = ElevationController::command_exists("ls");
        assert!(ls_exists);
    }

    #[test]
    fn test_elevation_manager_get_elevation_command() {
        let elevation = ElevationController::get_elevation_command();
        match elevation {
            Some((cmd, _args)) => {
                println!("Found elevation command: {}", cmd);
                assert!(cmd == "sudo" || cmd == "doas" || cmd == "su");
            }
            None => {
                println!("No elevation command found or already elevated");
            }
        }
    }

    #[test]
    fn test_command_builder_basic() {
        let result = CommandBuilder::new("echo")
            .arg("hello")
            .output_mode(OutputMode::Capture)
            .execute();

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.is_success());
        assert!(output.stdout.contains("hello"));
    }

    #[test]
    fn test_command_builder_with_args() {
        let cmd = "echo";
        let args = ["hello", "world"];

        let result = CommandBuilder::with_args(cmd, &args)
            .output_mode(OutputMode::Capture)
            .execute();

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.is_success());
        assert!(output.stdout.contains("hello world"));
    }

    #[test]
    fn test_command_builder_chained_args() {
        let base_cmd = "echo";
        let arg1 = "hello";
        let arg2 = "world";
        let additional_args = ["from", "rust"];

        let result = CommandBuilder::new(base_cmd)
            .arg(arg1)
            .arg(arg2)
            .args(&additional_args)
            .output_mode(OutputMode::Capture)
            .execute();

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.is_success());
        assert!(output.stdout.contains("hello world from rust"));
    }

    #[test]
    fn test_command_builder_with_env() {
        let cmd = "sh";
        let args = ["-c", "echo $TEST_VAR"];
        let env_key = "TEST_VAR";
        let env_value = "test_value";

        let result = CommandBuilder::with_args(cmd, &args)
            .env(env_key, env_value)
            .output_mode(OutputMode::Capture)
            .execute();

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.is_success());
        assert!(output.stdout.contains("test_value"));
    }

    #[test]
    fn test_command_builder_status_mode() {
        let result = CommandBuilder::new("false").execute_status();

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.is_success());
        assert_ne!(output.exit_code, 0);
    }

    #[test]
    fn test_command_executor_elevation_methods() {
        let executor = CommandExecutor::default();

        let is_elevated = executor.is_elevated();
        let elevation_available = executor.elevation_available();

        println!("Is elevated: {}", is_elevated);
        println!("Elevation available: {}", elevation_available);

        // Should not panic and return boolean values
        assert!(is_elevated == true || is_elevated == false);
        assert!(elevation_available == true || elevation_available == false);
    }

    #[test]
    fn test_privileged_execution_detection() {
        let result = CommandBuilder::new("whoami")
            .output_mode(OutputMode::Capture)
            .execute_privileged();

        match result {
            Ok(output) => {
                println!("Privileged command output: {}", output.stdout.trim());
                assert!(output.is_success());
            }
            Err(e) => {
                println!(
                    "Privileged execution failed (expected if no elevation available): {}",
                    e
                );
            }
        }
    }

    #[test]
    fn test_working_directory() {
        let working_dir = "/tmp";

        let result = CommandBuilder::new("pwd")
            .working_directory(working_dir)
            .output_mode(OutputMode::Capture)
            .execute();

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.is_success());
        assert!(output.stdout.contains("/tmp"));
    }

    #[test]
    fn test_multiple_env_vars() {
        let cmd = "sh";
        let script = "echo $VAR1 $VAR2";
        let args = ["-c", script];
        let env_vars = [("VAR1", "value1"), ("VAR2", "value2")];

        let result = CommandBuilder::with_args(cmd, &args)
            .envs(&env_vars)
            .output_mode(OutputMode::Capture)
            .execute();

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.is_success());
        assert!(output.stdout.contains("value1 value2"));
    }
}
