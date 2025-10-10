use anyhow::{Context, Result};
use log::{debug, info, warn};
use std::collections::HashMap;
#[cfg(not(target_os = "macos"))]
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const PKGX_BIN_PATHS: [&str; 2] = ["/usr/local/bin/pkgx", "/usr/local/bin/pkgm"];
#[cfg(target_os = "macos")]
const PKGX_MACOS_DATA_PATHS: [&str; 2] =
    ["Library/Caches/pkgx", "Library/Application Support/pkgx"];

pub struct RunConfig<'a> {
    pub tool: &'a str,
    pub args: Vec<String>,
    pub working_dir: &'a str,
    pub env_vars: Vec<String>,
    pub keep_package: bool,
    pub keep_pkgx: bool,
}

fn uninstall_pkgx() -> Result<()> {
    info!("Uninstalling pkgx and removing all associated files...");
    let items_to_delete = collect_pkgx_paths()?;
    if items_to_delete.is_empty() {
        info!("No pkgx installation found to remove");
        return Ok(());
    }

    info!("Found {} pkgx items to remove", items_to_delete.len());
    let mut failed_items = Vec::new();

    for path in items_to_delete {
        if !path.exists() {
            continue;
        }

        let result = if path.is_dir() {
            std::fs::remove_dir_all(&path)
        } else {
            std::fs::remove_file(&path)
        };

        match result {
            Ok(()) => {
                debug!("Removed: {}", path.display());
            }
            Err(e) => {
                let error_msg = format!("{}", e);
                warn!("Failed to remove {}: {}", path.display(), error_msg);
                failed_items.push((path.display().to_string(), error_msg));
            }
        }
    }

    if !failed_items.is_empty() {
        warn!("Failed to remove {} items:", failed_items.len());
        for (path, error) in failed_items {
            warn!("  - {}: {}", path, error);
        }
    }

    Ok(())
}

/// Collect all pkgx-related paths to delete based on the operating system
fn collect_pkgx_paths() -> Result<Vec<PathBuf>> {
    let mut paths: Vec<PathBuf> = Vec::new();
    for bin_path in PKGX_BIN_PATHS {
        paths.push(PathBuf::from(bin_path));
    }

    if let Some(home_dir) = dirs_next::home_dir() {
        paths.push(home_dir.join(".pkgx"));
    }

    let platform_paths = get_platform_specific_paths()?;
    for path in platform_paths {
        paths.push(path);
    }

    let existing_paths: Vec<PathBuf> = paths.into_iter().filter(|path| path.exists()).collect();

    Ok(existing_paths)
}

fn get_platform_specific_paths() -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();

    if let Some(home_dir) = dirs_next::home_dir() {
        #[cfg(target_os = "macos")]
        {
            for path in PKGX_MACOS_DATA_PATHS {
                paths.push(home_dir.join(path));
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            let cache_dir = if let Ok(xdg_cache) = env::var("XDG_CACHE_HOME") {
                PathBuf::from(xdg_cache)
            } else {
                home_dir.join(".cache")
            };
            paths.push(cache_dir.join("pkgx"));

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

pub fn execute(input: &RunConfig) -> Result<()> {
    let _lock = crate::utils::locking::acquire_lock().context("Failed to acquire lock")?;
    // TODO: Locking does not work correctly. Disabled for now.
    // let _lock = {
    //     let deadline = std::time::Instant::now() + std::time::Duration::from_secs(300); // 5 minutes
    //     loop {
    //         match utils::locking::acquire_lock() {
    //             Ok(lock) => break lock,
    //             Err(e) => {
    //                 if std::time::Instant::now() >= deadline {
    //                     return Err(anyhow::anyhow!(
    //                         "Failed to acquire lock within 5 minutes: {}",
    //                         e
    //                     ));
    //                 }
    //                 std::thread::sleep(std::time::Duration::from_millis(500));
    //             }
    //         }
    //     }
    // };
    // Example errors
    // - Failed to resolve package with libpkgx: Failed to install packages: No such file or directory (os error 2) at path "/home/vscode/.pkgx/python.org/.tmpA6vBNt"
    // - Could not find platform independent libraries <prefix> Fatal Python error:
    // - Failed to import encodings module ModuleNotFoundError: No module named 'encodings' pkgx library execution failed: Command failed with exit code: 1
    //
    // Test Failures:
    // - test_picolayer_run_multiple_args
    // - test_picolayer_run_node_version
    // - test_picolayer_run_node_with_version_simple
    // - test_picolayer_run_python_latest
    // - test_picolayer_run_python_script
    // - test_picolayer_run_python_version

    let working_path = Path::new(input.working_dir);
    if !working_path.exists() {
        anyhow::bail!("Working directory does not exist: {}", input.working_dir);
    }
    let (tool_name, version_spec) = parse_tool_spec(input.tool);
    info!("Working directory: {}", input.working_dir);
    info!("Tool: {} ({})", tool_name, version_spec);
    info!("Command: {}", input.args.join(" "));

    let mut env_map = Vec::new();
    for env_var in &input.env_vars {
        if let Some((key, value)) = env_var.split_once('=') {
            env_map.push((key.to_string(), value.to_string()));
        } else {
            anyhow::bail!(
                "Invalid environment variable format: {} (expected key=value)",
                env_var
            );
        }
    }

    let exec_result = if crate::utils::pkgx::check_pkgx_binary() {
        execute_with_pkgx_binary(
            &tool_name,
            &version_spec,
            &input.args,
            working_path,
            &env_map,
        )
    } else {
        execute_with_pkgx_library(
            &tool_name,
            &version_spec,
            &input.args,
            working_path,
            &env_map,
            input.keep_package,
        )
    };

    if !input.keep_pkgx {
        uninstall_pkgx()?;
    }

    match exec_result {
        Ok(()) => {}
        Err(e) => {
            warn!("Command failed: {}", e);
        }
    }

    Ok(())
}

fn parse_tool_spec(tool: &str) -> (String, String) {
    if let Some((name, version)) = tool.split_once('@') {
        (name.to_string(), version.to_string())
    } else {
        (tool.to_string(), "latest".to_string())
    }
}

fn execute_with_pkgx_library(
    tool_name: &str,
    version_spec: &str,
    args: &[String],
    working_path: &Path,
    env_map: &[(String, String)],
    keep_package: bool,
) -> Result<()> {
    info!("Using pkgx library integration...");

    match try_libpkgx_execution(
        tool_name,
        version_spec,
        args,
        working_path,
        env_map,
        keep_package,
    ) {
        Ok(()) => {
            info!("Command executed with pkgx library!");
            Ok(())
        }
        Err(e) => {
            warn!("pkgx library execution failed: {}", e);
            info!("Falling back to pkgx binary execution...");
            execute_with_pkgx_binary(tool_name, version_spec, args, working_path, env_map)
        }
    }
}

fn try_libpkgx_execution(
    tool_name: &str,
    version_spec: &str,
    args: &[String],
    working_path: &Path,
    env_map: &[(String, String)],
    keep_package: bool,
) -> Result<()> {
    use std::env;

    if args.is_empty() {
        anyhow::bail!("No arguments provided for tool: {}", tool_name);
    }

    // Resolve tool to project using pantry database
    let (project_name, tool_spec) = crate::utils::pkgx::resolve_tool_to_project(tool_name, version_spec)?;

    info!("Resolving package: {}", tool_spec);
    let mut cmd_env = HashMap::new();
    for (key, value) in env::vars() {
        // Overwrite Go environment variables that might be set by Mise or other shellenv tools
        if tool_name == "go" && (key == "GOROOT" || key == "GOPATH") {
            continue;
        }
        cmd_env.insert(key, value);
    }

    for (key, value) in env_map {
        cmd_env.insert(key.clone(), value.clone());
    }

    let mut paths_to_cleanup = Vec::new();

    let execution_result = match crate::utils::hydrate::resolve_package_with_libpkgx(&[tool_spec]) {
        Ok((pkgx_env, installations)) => {
            for (key, value) in pkgx_env {
                cmd_env.insert(key, value);
            }

            // Overwrite GOROOT for Go installations due to conflicts with Mise or other shellenv tools
            if tool_name == "go" {
                for installation in &installations {
                    if installation.pkg.project == project_name {
                        cmd_env.insert(
                            "GOROOT".to_string(),
                            installation.path.to_string_lossy().to_string(),
                        );
                        break;
                    }
                }
            }

            for installation in &installations {
                if installation.pkg.project == project_name {
                    info!("Package installed at: {}", installation.path.display());

                    if !keep_package {
                        paths_to_cleanup.push(installation.path.clone());
                    }

                    let bin_paths = vec!["bin", "sbin"];
                    for bin_dir in bin_paths {
                        let executable_path = installation.path.join(bin_dir).join(tool_name);
                        if executable_path.exists() {
                            info!("Executable found at: {}", executable_path.display());
                            break;
                        }
                    }
                }
            }

            info!("Resolved package with libpkgx");
            let mut cmd = Command::new(tool_name);
            cmd.args(args);
            cmd.current_dir(working_path);
            cmd.env_clear();
            for (key, value) in &cmd_env {
                cmd.env(key, value);
            }

            cmd.stdout(Stdio::inherit());
            cmd.stderr(Stdio::inherit());
            cmd.stdin(Stdio::inherit());

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
            warn!("Failed to resolve package with libpkgx: {}", e);
            Err(e)
        }
    };

    if !keep_package && !paths_to_cleanup.is_empty() {
        for path in paths_to_cleanup {
            if let Err(e) = cleanup_installation(&path) {
                warn!("Failed to cleanup {}: {}", path.display(), e);
            } else {
                info!("Removed: {}", path.display());
            }
        }
        info!("Cleanup completed");
    }

    execution_result
}

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

fn execute_with_pkgx_binary(
    tool_name: &str,
    version_spec: &str,
    args: &[String],
    working_path: &Path,
    env_map: &[(String, String)],
) -> Result<()> {
    let pkgx_available = crate::utils::pkgx::check_pkgx_binary();

    if !pkgx_available {
        anyhow::bail!("pkgx is not available. Install pkgx from https://pkgx.sh.");
    }

    let (project_name, _) = crate::utils::pkgx::resolve_tool_to_project(tool_name, version_spec)?;

    let mut cmd = Command::new("pkgx");

    if version_spec == "latest" {
        cmd.arg(format!("+{}", project_name));
    } else {
        cmd.arg(format!("+{}@{}", project_name, version_spec));
    }

    cmd.arg(tool_name);
    cmd.args(args);

    cmd.current_dir(working_path);

    for (key, value) in env_map {
        cmd.env(key, value);
    }

    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());
    cmd.stdin(Stdio::inherit());

    let status = cmd
        .status()
        .context("Failed to execute command with pkgx")?;

    if !status.success() {
        anyhow::bail!(
            "Command failed with exit code: {}",
            status.code().unwrap_or(-1)
        );
    }

    info!("Command executed successfully!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tool_spec_with_version() {
        let (name, version) = parse_tool_spec("python@3.11");
        assert_eq!(name, "python");
        assert_eq!(version, "3.11");
    }

    #[test]
    fn test_parse_tool_spec_without_version() {
        let (name, version) = parse_tool_spec("python");
        assert_eq!(name, "python");
        assert_eq!(version, "latest");
    }

    #[test]
    fn test_parse_tool_spec_complex_version() {
        let (name, version) = parse_tool_spec("node@18.16.0");
        assert_eq!(name, "node");
        assert_eq!(version, "18.16.0");
    }

    #[test]
    fn test_map_tool_to_project_python() {
        use libpkgx::{config::Config, sync};

        let config = Config::new().expect("Failed to initialize config");
        std::fs::create_dir_all(config.pantry_db_file.parent().unwrap()).unwrap();
        let mut conn = rusqlite::Connection::open(&config.pantry_db_file).unwrap();

        // Sync the database if needed
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if sync::should(&config).unwrap_or(false) {
                sync::ensure(&config, &mut conn).await.ok();
            }
        });

        // Test resolution for common tools
        let result = crate::utils::pkgx::map_tool_to_project("python", &conn);
        assert!(result.is_ok());
        // The result should be a valid project name (e.g., "python.org")
        let project = result.unwrap();
        assert!(!project.is_empty());
    }

    #[test]
    fn test_map_tool_to_project_node() {
        use libpkgx::{config::Config, sync};

        let config = Config::new().expect("Failed to initialize config");
        std::fs::create_dir_all(config.pantry_db_file.parent().unwrap()).unwrap();
        let mut conn = rusqlite::Connection::open(&config.pantry_db_file).unwrap();

        // Sync the database if needed
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if sync::should(&config).unwrap_or(false) {
                sync::ensure(&config, &mut conn).await.ok();
            }
        });

        let result = crate::utils::pkgx::map_tool_to_project("node", &conn);
        assert!(result.is_ok());
        let project = result.unwrap();
        assert!(!project.is_empty());
    }

    #[test]
    fn test_map_tool_to_project_go() {
        use libpkgx::{config::Config, sync};

        let config = Config::new().expect("Failed to initialize config");
        std::fs::create_dir_all(config.pantry_db_file.parent().unwrap()).unwrap();
        let mut conn = rusqlite::Connection::open(&config.pantry_db_file).unwrap();

        // Sync the database if needed
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if sync::should(&config).unwrap_or(false) {
                sync::ensure(&config, &mut conn).await.ok();
            }
        });

        let result = crate::utils::pkgx::map_tool_to_project("go", &conn);
        assert!(result.is_ok());
        let project = result.unwrap();
        assert!(!project.is_empty());
    }

    #[test]
    fn test_map_tool_to_project_rust() {
        use libpkgx::{config::Config, sync};

        let config = Config::new().expect("Failed to initialize config");
        std::fs::create_dir_all(config.pantry_db_file.parent().unwrap()).unwrap();
        let mut conn = rusqlite::Connection::open(&config.pantry_db_file).unwrap();

        // Sync the database if needed
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if sync::should(&config).unwrap_or(false) {
                sync::ensure(&config, &mut conn).await.ok();
            }
        });

        let result = crate::utils::pkgx::map_tool_to_project("cargo", &conn);
        assert!(result.is_ok());
        let project = result.unwrap();
        assert!(!project.is_empty());
    }

    #[test]
    fn test_map_tool_to_project_unknown() {
        use libpkgx::{config::Config, sync};

        let config = Config::new().expect("Failed to initialize config");
        std::fs::create_dir_all(config.pantry_db_file.parent().unwrap()).unwrap();
        let mut conn = rusqlite::Connection::open(&config.pantry_db_file).unwrap();

        // Sync the database if needed
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if sync::should(&config).unwrap_or(false) {
                sync::ensure(&config, &mut conn).await.ok();
            }
        });

        let result = crate::utils::pkgx::map_tool_to_project("unknown-tool-xyz-not-real", &conn);
        assert!(result.is_ok());
        // Unknown tools should fall back to the tool name itself
        let project = result.unwrap();
        assert_eq!(project, "unknown-tool-xyz-not-real");
    }

    #[test]
    fn test_resolve_tool_to_project() {
        // Test the full resolution flow including version spec
        let result = crate::utils::pkgx::resolve_tool_to_project("node", "latest");
        match &result {
            Ok(_) => {
                let (project, spec) = result.unwrap();
                assert!(!project.is_empty());
                assert_eq!(spec, project); // "latest" should just return the project name
            }
            Err(e) => {
                // If sync fails (e.g., network issue, 403 error), skip the test
                eprintln!("Skipping test due to sync error: {}", e);
                if e.to_string().contains("403 Forbidden") || e.to_string().contains("HTTP") {
                    // This is expected in CI/testing environments with rate limiting
                    return;
                }
                panic!("Unexpected error: {}", e);
            }
        }

        // Test with version
        let result = crate::utils::pkgx::resolve_tool_to_project("python", "3.11");
        match &result {
            Ok(_) => {
                let (project, spec) = result.unwrap();
                assert!(!project.is_empty());
                assert!(spec.contains("@3.11"));
            }
            Err(e) => {
                eprintln!("Skipping test due to sync error: {}", e);
                if e.to_string().contains("403 Forbidden") || e.to_string().contains("HTTP") {
                    return;
                }
                panic!("Unexpected error: {}", e);
            }
        }
    }

    #[test]
    fn test_query_various_tools() {
        use libpkgx::{config::Config, sync};

        let config = Config::new().expect("Failed to initialize config");
        std::fs::create_dir_all(config.pantry_db_file.parent().unwrap()).unwrap();
        let mut conn = rusqlite::Connection::open(&config.pantry_db_file).unwrap();

        // Sync the database if needed
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if sync::should(&config).unwrap_or(false) {
                sync::ensure(&config, &mut conn).await.ok();
            }
        });

        // Test various common tools
        let tools = vec!["bash", "git", "curl", "wget", "make"];
        for tool in tools {
            let result = crate::utils::pkgx::map_tool_to_project(tool, &conn);
            assert!(result.is_ok(), "Failed to query tool: {}", tool);
            let project = result.unwrap();
            assert!(!project.is_empty(), "Empty project for tool: {}", tool);
        }
    }

    #[test]
    fn test_collect_pkgx_paths() {
        let paths = collect_pkgx_paths();

        assert!(paths.is_ok());

        let paths = paths.unwrap();
        let path_strings: Vec<String> = paths.iter().map(|p| p.display().to_string()).collect();

        let _has_bin_paths = path_strings
            .iter()
            .any(|p| p.contains("/usr/local/bin/pkgx"));
        let _has_home_path = path_strings.iter().any(|p| p.contains(".pkgx"));
    }

    #[test]
    fn test_get_platform_specific_paths() {
        let paths = get_platform_specific_paths();

        assert!(paths.is_ok());

        let paths = paths.unwrap();

        assert!(paths.len() >= 1);

        for path in paths {
            assert!(path.to_string_lossy().contains("pkgx"));
        }
    }
}
