use anyhow::{Context, Result};
#[cfg(not(target_os = "macos"))]
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[cfg(feature = "pkgx-integration")]
use std::collections::HashMap;

/// Completely uninstall pkgx and remove all associated files
pub fn uninstall_pkgx() -> Result<()> {
    println!("Uninstalling pkgx and removing all associated files...");

    let mut removed_count = 0;
    let mut error_count = 0;

    // 1. Remove pkgx binaries from /usr/local/bin
    let bin_paths = vec!["/usr/local/bin/pkgx", "/usr/local/bin/pkgm"];

    for bin_path in bin_paths {
        let path = PathBuf::from(bin_path);
        if path.exists() {
            match std::fs::remove_file(&path) {
                Ok(()) => {
                    println!("Removed binary: {}", path.display());
                    removed_count += 1;
                }
                Err(e) => {
                    eprintln!("Failed to remove {}: {} (try with sudo)", path.display(), e);
                    error_count += 1;
                }
            }
        }
    }

    // 2. Remove main pkgx directory (~/.pkgx)
    if let Some(home_dir) = dirs_next::home_dir() {
        let pkgx_dir = home_dir.join(".pkgx");
        if pkgx_dir.exists() {
            match std::fs::remove_dir_all(&pkgx_dir) {
                Ok(()) => {
                    println!("Removed main directory: {}", pkgx_dir.display());
                    removed_count += 1;
                }
                Err(e) => {
                    eprintln!("Failed to remove {}: {}", pkgx_dir.display(), e);
                    error_count += 1;
                }
            }
        }
    }

    // 3. Remove platform-specific cache and data directories
    let cache_data_paths = get_platform_specific_paths()?;

    for path in cache_data_paths {
        if path.exists() {
            let result = if path.is_dir() {
                std::fs::remove_dir_all(&path)
            } else {
                std::fs::remove_file(&path)
            };

            match result {
                Ok(()) => {
                    println!("Removed cache/data: {}", path.display());
                    removed_count += 1;
                }
                Err(e) => {
                    eprintln!("Failed to remove {}: {}", path.display(), e);
                    error_count += 1;
                }
            }
        }
    }

    // Summary
    println!("Uninstall Summary:");
    println!("Successfully removed: {} items", removed_count);
    if error_count > 0 {
        println!("Failed to remove: {} items", error_count);
        println!("Some files may require sudo privileges to remove");
    }

    if removed_count > 0 {
        println!("pkgx uninstallation completed!");
    } else {
        println!("No pkgx installation found to remove");
    }

    Ok(())
}

fn get_platform_specific_paths() -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();

    if let Some(home_dir) = dirs_next::home_dir() {
        #[cfg(target_os = "macos")]
        {
            // macOS specific paths
            paths.push(home_dir.join("Library/Caches/pkgx"));
            paths.push(home_dir.join("Library/Application Support/pkgx"));
        }

        #[cfg(not(target_os = "macos"))]
        {
            // Non-macOS (Linux, etc.) paths
            // XDG_CACHE_HOME or fallback to ~/.cache
            let cache_dir = if let Ok(xdg_cache) = env::var("XDG_CACHE_HOME") {
                PathBuf::from(xdg_cache)
            } else {
                home_dir.join(".cache")
            };
            paths.push(cache_dir.join("pkgx"));

            // XDG_DATA_HOME or fallback to ~/.local/share
            let data_dir = if let Ok(xdg_data) = env::var("XDG_DATA_HOME") {
                PathBuf::from(xdg_data)
            } else {
                home_dir.join(".local/share")
            };
            paths.push(data_dir.join("pkgx"));
        }
    }

    Ok(paths)
}

/// Execute a command using pkgx for dependency management
pub fn execute(
    tool: &str,
    args: &[String],
    working_dir: &str,
    env_vars: &[String],
    _force_pkgx: bool,
    ephemeral: bool,
) -> Result<()> {
    // Change to working directory
    let working_path = Path::new(working_dir);
    if !working_path.exists() {
        anyhow::bail!("Working directory does not exist: {}", working_dir);
    }

    // Parse tool specification (e.g., "python@3.10" or "python")
    let (tool_name, version_spec) = parse_tool_spec(tool);

    println!(
        "Executing: {} with tool: {} ({})",
        args.join(" "),
        tool_name,
        version_spec
    );
    println!("Working directory: {}", working_dir);

    if ephemeral {
        println!("Ephemeral mode: packages will be removed after execution");
    }

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
            return execute_with_pkgx_library(
                &tool_name,
                &version_spec,
                args,
                working_path,
                &env_map,
                ephemeral,
            );
        }
    }

    execute_with_pkgx_binary(&tool_name, &version_spec, args, working_path, &env_map)
}

fn parse_tool_spec(tool: &str) -> (String, String) {
    if let Some((name, version)) = tool.split_once('@') {
        (name.to_string(), version.to_string())
    } else {
        (tool.to_string(), "latest".to_string())
    }
}

// Map tool name to proper project name
fn map_tool_to_project(tool_name: &str) -> String {
    match tool_name {
        "python" | "python3" | "pip" | "pip3" => "python.org".to_string(),
        "node" | "npm" | "npx" | "yarn" => "nodejs.org".to_string(),
        "ruby" | "gem" | "bundle" => "ruby-lang.org".to_string(),
        "go" | "gofmt" => "go.dev".to_string(),
        "java" | "javac" | "mvn" | "gradle" => "oracle.com/java".to_string(),
        "cargo" | "rustc" => "rust-lang.org".to_string(),
        "php" | "composer" => "php.net".to_string(),
        "docker" => "docker.com".to_string(),
        "git" => "git-scm.org".to_string(),
        "deno" => "deno.land".to_string(),
        "bun" => "bun.sh".to_string(),
        _ => {
            // For unknown tools, assume it might already be a project name
            tool_name.to_string() // Use as-is
        }
    }
}

#[cfg(feature = "pkgx-integration")]
fn execute_with_pkgx_library(
    tool_name: &str,
    version_spec: &str,
    args: &[String],
    working_path: &Path,
    env_map: &[(String, String)],
    ephemeral: bool,
) -> Result<()> {
    println!("Using pkgx library integration...");

    // Try to use libpkgx functionality with fallback to binary
    match try_libpkgx_execution(
        tool_name,
        version_spec,
        args,
        working_path,
        env_map,
        ephemeral,
    ) {
        Ok(()) => {
            println!("Command executed successfully with pkgx library!");
            Ok(())
        }
        Err(e) => {
            eprintln!("pkgx library execution failed: {}", e);
            eprintln!("Falling back to pkgx binary execution...");
            execute_with_pkgx_binary(tool_name, version_spec, args, working_path, env_map)
        }
    }
}

#[cfg(feature = "pkgx-integration")]
fn try_libpkgx_execution(
    tool_name: &str,
    version_spec: &str,
    args: &[String],
    working_path: &Path,
    env_map: &[(String, String)],
    ephemeral: bool,
) -> Result<()> {
    use std::env;

    if args.is_empty() {
        anyhow::bail!("No arguments provided for tool: {}", tool_name);
    }

    // Map tool name to proper project name
    let project_name = map_tool_to_project(tool_name);

    // Create tool specification for pkgx
    let tool_spec = if version_spec == "latest" {
        project_name.clone()
    } else {
        format!("{}@{}", project_name, version_spec)
    };

    println!("Resolving package: {}", tool_spec);

    // Set up environment with current variables
    let mut cmd_env = HashMap::new();

    // Get current environment
    for (key, value) in env::vars() {
        cmd_env.insert(key, value);
    }

    // Add user-provided environment variables
    for (key, value) in env_map {
        cmd_env.insert(key.clone(), value.clone());
    }

    // Resolve the single tool with libpkgx and get installation info
    let mut paths_to_cleanup = Vec::new();

    let execution_result = match resolve_package_with_libpkgx(&[tool_spec]) {
        Ok((pkgx_env, installations)) => {
            // Merge pkgx environment with existing environment
            for (key, value) in pkgx_env {
                cmd_env.insert(key, value);
            }

            // Show installation paths and collect cleanup paths if ephemeral
            for installation in &installations {
                if installation.pkg.project == project_name {
                    println!("Package installed at: {}", installation.path.display());

                    if ephemeral {
                        paths_to_cleanup.push(installation.path.clone());
                    }

                    // Try to find the specific executable
                    let bin_paths = vec!["bin", "sbin"];
                    for bin_dir in bin_paths {
                        let executable_path = installation.path.join(bin_dir).join(tool_name);
                        if executable_path.exists() {
                            println!("Executable found at: {}", executable_path.display());
                            break;
                        }
                    }
                }
            }

            println!("Successfully resolved package with libpkgx");

            // Execute the command
            let mut cmd = Command::new(tool_name);
            cmd.args(args);

            cmd.current_dir(working_path);

            // Set environment variables
            cmd.env_clear();
            for (key, value) in &cmd_env {
                cmd.env(key, value);
            }

            // Inherit stdio for interactive execution
            cmd.stdout(Stdio::inherit());
            cmd.stderr(Stdio::inherit());
            cmd.stdin(Stdio::inherit());

            // Execute the command
            let status = cmd
                .status()
                .context("Failed to execute command with libpkgx")?;

            if !status.success() {
                anyhow::bail!(
                    "Command failed with exit code: {}",
                    status.code().unwrap_or(-1)
                );
            }

            Ok(())
        }
        Err(e) => {
            eprintln!("Warning: Failed to resolve package with libpkgx: {}", e);
            Err(e)
        }
    };

    // Cleanup ephemeral installations regardless of execution result
    if ephemeral && !paths_to_cleanup.is_empty() {
        println!("Cleaning up ephemeral installations...");
        for path in paths_to_cleanup {
            if let Err(e) = cleanup_installation(&path) {
                eprintln!("Warning: Failed to cleanup {}: {}", path.display(), e);
            } else {
                println!("Removed: {}", path.display());
            }
        }
        println!("Cleanup completed");
    }

    execution_result
}

#[cfg(feature = "pkgx-integration")]
fn cleanup_installation(path: &PathBuf) -> Result<()> {
    if path.exists() {
        if path.is_dir() {
            std::fs::remove_dir_all(path)
                .with_context(|| format!("Failed to remove directory: {}", path.display()))?;
        } else {
            std::fs::remove_file(path)
                .with_context(|| format!("Failed to remove file: {}", path.display()))?;
        }
    }
    Ok(())
}

#[cfg(feature = "pkgx-integration")]
fn resolve_package_with_libpkgx(
    dependencies: &[String],
) -> Result<(HashMap<String, String>, Vec<libpkgx::types::Installation>)> {
    // Create a Tokio runtime for async libpkgx operations
    let rt = tokio::runtime::Runtime::new()
        .context("Failed to create Tokio runtime for libpkgx operations")?;

    rt.block_on(async { resolve_dependencies_async(dependencies).await })
}

#[cfg(feature = "pkgx-integration")]
async fn resolve_dependencies_async(
    dependencies: &[String],
) -> Result<(HashMap<String, String>, Vec<libpkgx::types::Installation>)> {
    use libpkgx::{
        config::Config, hydrate, install_multi::ProgressBarExt, pantry_db, resolve, sync,
        types::PackageReq,
    };
    use std::collections::HashMap;
    use std::sync::Arc;

    // Real progress bar implementation
    struct ToolProgressBar {
        bar: indicatif::ProgressBar,
    }

    impl ToolProgressBar {
        fn new() -> Self {
            let bar = indicatif::ProgressBar::new(0);
            bar.set_style(
                indicatif::ProgressStyle::with_template(
                    "{elapsed:.dim} ❲{wide_bar:.cyan/blue}❳ {percent}% {bytes_per_sec:.dim} {bytes:.dim}"
                ).unwrap()
                .progress_chars("██░")
            );
            Self { bar }
        }
    }

    impl ProgressBarExt for ToolProgressBar {
        fn inc(&self, n: u64) {
            self.bar.inc(n);
        }

        fn inc_length(&self, n: u64) {
            self.bar.inc_length(n);
        }
    }

    // Initialize libpkgx configuration
    let config = Config::new().context("Failed to initialize libpkgx config")?;

    // Create database directory if it doesn't exist
    std::fs::create_dir_all(config.pantry_db_file.parent().unwrap())?;
    let mut conn = rusqlite::Connection::open(&config.pantry_db_file)?;

    // Sync pantry database if needed
    if sync::should(&config).map_err(|e| anyhow::anyhow!("{}", e))? {
        println!("Syncing pkgx pantry database...");
        sync::ensure(&config, &mut conn)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
    }

    let mut resolved_env = HashMap::new();

    // Convert dependency strings to PackageReq objects
    let mut package_reqs = Vec::new();
    for dep in dependencies {
        match PackageReq::parse(dep) {
            Ok(req) => package_reqs.push(req),
            Err(e) => {
                eprintln!("Warning: Failed to parse dependency {}: {}", dep, e);
                continue;
            }
        }
    }

    if package_reqs.is_empty() {
        return Ok((resolved_env, Vec::new()));
    }

    // Hydrate dependencies (resolve dependency graph)
    let hydrated_packages = hydrate::hydrate(&package_reqs, |project| {
        pantry_db::deps_for_project(&project, &conn)
    })
    .await
    .map_err(|e| anyhow::anyhow!("Failed to hydrate dependencies: {}", e))?;

    // Resolve packages to specific versions and installations
    let resolution = resolve::resolve(&hydrated_packages, &config)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to resolve packages: {}", e))?;

    // Install any pending packages
    let mut installations = resolution.installed;
    if !resolution.pending.is_empty() {
        println!(
            "Installing {} packages with libpkgx...",
            resolution.pending.len()
        );
        let progress_bar = ToolProgressBar::new();
        let installed = libpkgx::install_multi::install_multi(
            &resolution.pending,
            &config,
            Some(Arc::new(progress_bar)),
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to install packages: {}", e))?;
        installations.extend(installed);
    }

    // Get environment variables from installed packages
    let env_map = libpkgx::env::map(&installations);
    let platform_env = libpkgx::env::mix(env_map);
    let runtime_env = libpkgx::env::mix_runtime(&platform_env, &installations, &conn)
        .map_err(|e| anyhow::anyhow!("Failed to mix runtime environment: {}", e))?;

    // Convert platform-aware environment keys to strings
    for (key, value) in runtime_env {
        // Remove shell variable expansion syntax for direct execution
        let cleaned_value = clean_shell_expansion(&value, &key.to_string());
        resolved_env.insert(key.to_string(), cleaned_value);
    }

    println!(
        "Successfully resolved {} packages with libpkgx",
        dependencies.len()
    );
    Ok((resolved_env, installations))
}

#[cfg(feature = "pkgx-integration")]
fn clean_shell_expansion(value: &str, key: &str) -> String {
    use regex::Regex;

    // Remove ${KEY:-default} syntax and just use the value part
    let re = Regex::new(&format!(r"\$\{{{key}:-([^}}]+)\}}")).unwrap();
    if let Some(caps) = re.captures(value) {
        return caps.get(1).unwrap().as_str().to_string();
    }

    // Remove ${KEY:+:${KEY}} syntax
    let re2 = Regex::new(&format!(r"\$\{{{key}:\+:[^}}]*\}}")).unwrap();
    let cleaned = re2.replace_all(value, "");

    // Remove trailing :${KEY} references
    let re3 = Regex::new(&format!(r":?\$\{{{key}\}}$")).unwrap();
    let cleaned = re3.replace_all(&cleaned, "");

    // Remove leading :${KEY}: references
    let re4 = Regex::new(&format!(r"^:?\$\{{{key}\}}:?")).unwrap();
    let cleaned = re4.replace_all(&cleaned, "");

    cleaned.to_string()
}

fn execute_with_pkgx_binary(
    tool_name: &str,
    version_spec: &str,
    args: &[String],
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

    // Map tool name to project name for binary execution too
    let project_name = map_tool_to_project(tool_name);

    // Execute command with pkgx
    let mut cmd = Command::new("pkgx");

    // Add tool specification
    if version_spec == "latest" {
        cmd.arg(format!("+{}", project_name));
    } else {
        cmd.arg(format!("+{}@{}", project_name, version_spec));
    }

    // Add the actual command
    cmd.arg(tool_name);
    cmd.args(args);

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
