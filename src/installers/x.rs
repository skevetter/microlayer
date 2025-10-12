use anyhow::{Context, Result};
use log::info;
use std::path::Path;
use std::{collections::HashMap, env};
use tempfile::TempDir;

use crate::utils::pkgx;

pub struct RunConfig<'a> {
    pub tool: &'a str,
    pub args: Vec<String>,
    pub working_dir: &'a str,
    pub env_vars: Vec<String>,
}

pub fn execute(input: &RunConfig) -> Result<()> {
    validate_working_directory(input.working_dir)?;
    let (tool_name, version_spec) = parse_tool_spec(input.tool);
    info!("Working directory: {}", input.working_dir);
    info!("Tool: {} ({})", tool_name, version_spec);
    info!("Command: {}", input.args.join(" "));

    let env_map = parse_env_vars(&input.env_vars)?;

    let _temp_dir =
        TempDir::with_prefix("picolayer_").context("Failed to create temporary directory")?;

    // Create virtual environment directory structure
    let pkgx_dir = _temp_dir.path().join("x").join("pkgx");
    let pantry_dir = _temp_dir.path().join("x").join("pantry");

    // Set PKGX_DIR and PKGX_PANTRY_DIR for for pantry isolation
    unsafe {
        std::env::set_var("PKGX_DIR", &pkgx_dir);
    }
    unsafe {
        std::env::set_var("PKGX_PANTRY_DIR", &pantry_dir);
    }

    // Ensure directories exist
    std::fs::create_dir_all(&pkgx_dir).context("Failed to create pkgx directory")?;
    std::fs::create_dir_all(&pantry_dir).context("Failed to create pantry directory")?;

    let pkgx_dir_str = pkgx_dir
        .to_str()
        .context("Failed to convert pkgx directory path to string")?;

    let pantry_dir_str = pantry_dir
        .to_str()
        .context("Failed to convert pantry directory path to string")?;

    info!("Using pkgx virtual environment: {}", pkgx_dir_str);
    info!("Using pantry directory: {}", pantry_dir_str);

    let working_path = Path::new(input.working_dir);

    if pkgx::check_pkgx_binary() {
        execute_with_pkgx_binary(
            &tool_name,
            &version_spec,
            &input.args,
            working_path,
            &env_map,
            pkgx_dir_str,
            pantry_dir_str,
        )
    } else {
        execute_with_pkgx_library(
            &tool_name,
            &version_spec,
            &input.args,
            working_path,
            &env_map,
            pkgx_dir_str,
            pantry_dir_str,
        )
    }
}

fn validate_working_directory(working_dir: &str) -> Result<()> {
    let working_path = Path::new(working_dir);
    if !working_path.exists() {
        anyhow::bail!("Working directory does not exist: {}", working_dir);
    }
    Ok(())
}

fn parse_env_vars(env_vars: &[String]) -> Result<Vec<(String, String)>> {
    env_vars
        .iter()
        .map(|env_var| {
            env_var
                .split_once('=')
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .context(format!(
                    "Invalid environment variable format: {} (expected key=value)",
                    env_var
                ))
        })
        .collect()
}

fn parse_tool_spec(tool: &str) -> (String, String) {
    match tool.split_once('@') {
        Some((name, version)) => (name.to_string(), version.to_string()),
        None => (tool.to_string(), "latest".to_string()),
    }
}

fn create_command_env(
    env_map: &[(String, String)],
    pkgx_dir: &str,
    pantry_dir: &str,
) -> HashMap<String, String> {
    let mut cmd_env: HashMap<String, String> = env::vars().collect();

    // https://docs.pkgx.sh/pkgx/pkgx#virtual-environments
    cmd_env.insert("PKGX_DIR".to_string(), pkgx_dir.to_string());

    // Set pantry directory for complete isolation
    cmd_env.insert("PKGX_PANTRY_DIR".to_string(), pantry_dir.to_string());

    // User provided environment variables
    for (key, value) in env_map {
        cmd_env.insert(key.clone(), value.clone());
    }

    cmd_env
}

fn execute_with_pkgx_library(
    tool_name: &str,
    version_spec: &str,
    args: &[String],
    working_path: &Path,
    env_map: &[(String, String)],
    pkgx_dir: &str,
    pantry_dir: &str,
) -> Result<()> {
    info!("Using pkgx library integration with virtual environment...");

    if args.is_empty() {
        anyhow::bail!("No arguments provided for tool: {}", tool_name);
    }

    let (project_name, tool_spec) = pkgx::resolve_tool_to_project(tool_name, version_spec)?;

    info!("Resolving package: {}", tool_spec);

    let mut cmd_env = create_command_env(env_map, pkgx_dir, pantry_dir);

    match pkgx::resolve_package_with_libpkgx(&[tool_spec]) {
        Ok((pkgx_env, installations)) => {
            cmd_env.extend(pkgx_env); // Merge pkgx environment variables

            for installation in &installations {
                if installation.pkg.project == project_name {
                    info!("Package installed at: {}", installation.path.display());

                    // Check for executable in common binary directories
                    for bin_dir in ["bin", "sbin"] {
                        let executable_path = installation.path.join(bin_dir).join(tool_name);
                        if executable_path.exists() {
                            info!("Executable found at: {}", executable_path.display());
                            break;
                        }
                    }
                }
            }

            info!("Resolved package with libpkgx");
            let status = std::process::Command::new(tool_name)
                .args(args)
                .current_dir(working_path.to_str().context("Invalid working directory")?)
                .envs(&cmd_env)
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .status()
                .context("Failed to execute command with libpkgx")?;

            if status.success() {
                info!("Command executed successfully with pkgx library!");
                Ok(())
            } else {
                anyhow::bail!("Command failed with exit code: {:?}", status.code());
            }
        }
        Err(e) => {
            info!(
                "Failed to resolve package with libpkgx, trying pkgx binary: {}",
                e
            );
            execute_with_pkgx_binary(
                tool_name,
                version_spec,
                args,
                working_path,
                env_map,
                pkgx_dir,
                pantry_dir,
            )
        }
    }
}

fn execute_with_pkgx_binary(
    tool_name: &str,
    version_spec: &str,
    args: &[String],
    working_path: &Path,
    env_map: &[(String, String)],
    pkgx_dir: &str,
    pantry_dir: &str,
) -> Result<()> {
    if !pkgx::check_pkgx_binary() {
        anyhow::bail!("pkgx is not available. Install pkgx from https://pkgx.sh.");
    }

    let (project_name, _) = pkgx::resolve_tool_to_project(tool_name, version_spec)?;

    let project_arg = if version_spec == "latest" {
        format!("+{}", project_name)
    } else {
        format!("+{}@{}", project_name, version_spec)
    };

    info!("Using pkgx binary with virtual environment...");

    let mut cmd = std::process::Command::new("pkgx");
    cmd.arg(&project_arg)
        .arg(tool_name)
        .args(args)
        .current_dir(working_path.to_str().context("Invalid working directory")?)
        .env("PKGX_DIR", pkgx_dir) // Set virtual environment directory
        .env("PKGX_PANTRY_DIR", pantry_dir); // Set pantry directory

    // User-provided environment variables
    for (key, value) in env_map {
        cmd.env(key, value);
    }

    let status = cmd
        .status()
        .context("Failed to execute command with pkgx")?;

    if status.success() {
        info!("Command executed successfully with pkgx binary!");
        Ok(())
    } else {
        anyhow::bail!("Command failed with exit code: {:?}", status.code());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_parse_tool_spec_with_version() {
        let (name, version) = parse_tool_spec("python@3.11");
        assert_eq!(name, "python");
        assert_eq!(version, "3.11");
    }

    #[test]
    #[serial]
    fn test_parse_tool_spec_without_version() {
        let (name, version) = parse_tool_spec("python");
        assert_eq!(name, "python");
        assert_eq!(version, "latest");
    }

    #[test]
    #[serial]
    fn test_parse_tool_spec_complex_version() {
        let (name, version) = parse_tool_spec("node@18.16.0");
        assert_eq!(name, "node");
        assert_eq!(version, "18.16.0");
    }

    #[test]
    #[serial]
    fn test_map_tool_to_project_python() {
        use libpkgx::{config::Config, sync};

        let config = Config::new().expect("Failed to initialize config");
        std::fs::create_dir_all(config.pantry_db_file.parent().unwrap()).unwrap();
        let mut conn = rusqlite::Connection::open(&config.pantry_db_file).unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if sync::should(&config).unwrap_or(false) {
                sync::ensure(&config, &mut conn).await.ok();
            }
        });

        let result = pkgx::map_tool_to_project("python", &conn);
        assert!(result.is_ok());
        let project = result.unwrap();
        assert!(!project.is_empty());
    }

    #[test]
    #[serial]
    fn test_map_tool_to_project_node() {
        use libpkgx::{config::Config, sync};

        let config = Config::new().expect("Failed to initialize config");
        std::fs::create_dir_all(config.pantry_db_file.parent().unwrap()).unwrap();
        let mut conn = rusqlite::Connection::open(&config.pantry_db_file).unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if sync::should(&config).unwrap_or(false) {
                sync::ensure(&config, &mut conn).await.ok();
            }
        });

        let result = pkgx::map_tool_to_project("node", &conn);
        assert!(result.is_ok());
        let project = result.unwrap();
        assert!(!project.is_empty());
    }

    #[test]
    #[serial]
    fn test_map_tool_to_project_go() {
        use libpkgx::{config::Config, sync};

        let config = Config::new().expect("Failed to initialize config");
        std::fs::create_dir_all(config.pantry_db_file.parent().unwrap()).unwrap();
        let mut conn = rusqlite::Connection::open(&config.pantry_db_file).unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if sync::should(&config).unwrap_or(false) {
                sync::ensure(&config, &mut conn).await.ok();
            }
        });

        let result = pkgx::map_tool_to_project("go", &conn);
        assert!(result.is_ok());
        let project = result.unwrap();
        assert!(!project.is_empty());
    }

    #[test]
    #[serial]
    fn test_map_tool_to_project_rust() {
        use libpkgx::{config::Config, sync};

        let config = Config::new().expect("Failed to initialize config");
        std::fs::create_dir_all(config.pantry_db_file.parent().unwrap()).unwrap();
        let mut conn = rusqlite::Connection::open(&config.pantry_db_file).unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if sync::should(&config).unwrap_or(false) {
                sync::ensure(&config, &mut conn).await.ok();
            }
        });

        let result = pkgx::map_tool_to_project("cargo", &conn);
        assert!(result.is_ok());
        let project = result.unwrap();
        assert!(!project.is_empty());
    }

    #[test]
    #[serial]
    fn test_map_tool_to_project_unknown() {
        use libpkgx::{config::Config, sync};

        let config = Config::new().expect("Failed to initialize config");
        std::fs::create_dir_all(config.pantry_db_file.parent().unwrap()).unwrap();
        let mut conn = rusqlite::Connection::open(&config.pantry_db_file).unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if sync::should(&config).unwrap_or(false) {
                sync::ensure(&config, &mut conn).await.ok();
            }
        });

        let result = pkgx::map_tool_to_project("unknown-tool-xyz-not-real", &conn);
        assert!(result.is_ok());
        let project = result.unwrap();
        assert_eq!(project, "unknown-tool-xyz-not-real");
    }

    #[test]
    #[serial]
    fn test_resolve_tool_to_project() {
        let result = pkgx::resolve_tool_to_project("node", "latest");
        match &result {
            Ok(_) => {
                let (project, spec) = result.unwrap();
                assert!(!project.is_empty());
                assert_eq!(spec, project);
            }
            Err(e) => {
                eprintln!("Skipping test due to sync error: {}", e);
                if e.to_string().contains("403 Forbidden") || e.to_string().contains("HTTP") {
                    return;
                }
                panic!("Unexpected error: {}", e);
            }
        }

        // Test with version
        let result = pkgx::resolve_tool_to_project("python", "3.11");
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
    #[serial]
    fn test_query_various_tools() {
        use libpkgx::{config::Config, sync};

        let config = Config::new().expect("Failed to initialize config");
        std::fs::create_dir_all(config.pantry_db_file.parent().unwrap()).unwrap();
        let mut conn = rusqlite::Connection::open(&config.pantry_db_file).unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if sync::should(&config).unwrap_or(false) {
                sync::ensure(&config, &mut conn).await.ok();
            }
        });

        let tools = vec!["bash", "git", "curl", "wget", "make"];
        for tool in tools {
            let result = pkgx::map_tool_to_project(tool, &conn);
            assert!(result.is_ok(), "Failed to query tool: {}", tool);
            let project = result.unwrap();
            assert!(!project.is_empty(), "Empty project for tool: {}", tool);
        }
    }
}
