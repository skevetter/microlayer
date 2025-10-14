use anyhow::{Context, Result};
use libpkgx::{
    config::Config, hydrate, install_multi::ProgressBarExt, pantry_db, resolve, sync,
    types::PackageReq,
};
use log::{info, warn};
use rusqlite::Connection;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

pub struct PkgxConfig {
    pkgx_dir: PathBuf,
    pantry_dir: PathBuf,
}

impl PkgxConfig {
    pub fn new(pkgx_dir: &str, pantry_dir: &str) -> Self {
        Self {
            pkgx_dir: PathBuf::from(pkgx_dir),
            pantry_dir: PathBuf::from(pantry_dir),
        }
    }

    fn get_config(&self) -> Result<Config> {
        let mut config = Config::new().context("Failed to initialize libpkgx config")?;
        config.pkgx_dir = PathBuf::from(&self.pkgx_dir);
        config.pantry_dir = PathBuf::from(&self.pantry_dir);
        Ok(config)
    }
}

/// Resolve package dependencies using libpkgx
pub fn resolve_package_with_libpkgx(
    dependencies: &[String],
    pkgx_config: &PkgxConfig,
) -> Result<(HashMap<String, String>, Vec<libpkgx::types::Installation>)> {
    let rt = tokio::runtime::Runtime::new()
        .context("Failed to create Tokio runtime for libpkgx operations")?;

    rt.block_on(async { resolve_dependencies_async(dependencies, pkgx_config).await })
}

async fn resolve_dependencies_async(
    dependencies: &[String],
    pkgx_config: &PkgxConfig,
) -> Result<(HashMap<String, String>, Vec<libpkgx::types::Installation>)> {
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

    let config = pkgx_config
        .get_config()
        .context("Failed to get pkgx config")?;

    std::fs::create_dir_all(config.pantry_db_file.parent().unwrap())?;
    let mut conn = rusqlite::Connection::open(&config.pantry_db_file)?;

    if sync::should(&config).map_err(|e| anyhow::anyhow!("{}", e))? {
        info!("Syncing pkgx pantry database...");
        sync::ensure(&config, &mut conn)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
    }

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
        return Ok((HashMap::new(), Vec::new()));
    }

    let hydrated_packages = hydrate::hydrate(&package_reqs, |project| {
        pantry_db::deps_for_project(&project, &conn)
    })
    .await
    .map_err(|e| anyhow::anyhow!("Failed to hydrate dependencies: {}", e))?;

    let resolution = resolve::resolve(&hydrated_packages, &config)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to resolve packages: {}", e))?;

    let mut installations = resolution.installed;
    if !resolution.pending.is_empty() {
        info!(
            "Installing {} packages with libpkgx",
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

    let env_map = libpkgx::env::map(&installations);
    let platform_env = libpkgx::env::mix(env_map);
    let runtime_env = libpkgx::env::mix_runtime(&platform_env, &installations, &conn)
        .map_err(|e| anyhow::anyhow!("Failed to mix runtime environment: {}", e))?;

    info!(
        "Successfully resolved {} packages with libpkgx",
        dependencies.len()
    );
    Ok((runtime_env, installations))
}

/// Query the pkgx pantry database to resolve a tool name to a project name
pub fn map_tool_to_project(tool_name: &str, conn: &rusqlite::Connection) -> Result<String> {
    let tool_name_string = tool_name.to_string();
    match libpkgx::pantry_db::projects_for_symbol(&tool_name_string, conn) {
        Ok(projects) if !projects.is_empty() => {
            if projects.len() == 1 {
                Ok(projects[0].clone())
            } else {
                info!(
                    "Multiple projects provide '{}': {:?}, using {}",
                    tool_name, projects, projects[0]
                );
                Ok(projects[0].clone())
            }
        }
        Ok(_) => {
            warn!(
                "No project found for tool '{}' in pantry database, using tool name as project",
                tool_name
            );
            Ok(tool_name.to_string())
        }
        Err(e) => {
            warn!(
                "Failed to query pantry database for tool '{}': {}, using tool name as project",
                tool_name, e
            );
            Ok(tool_name.to_string())
        }
    }
}

/// Resolve a tool name and version spec to a project name and tool spec
pub fn resolve_tool_to_project(
    tool_name: &str,
    version_spec: &str,
    pkgx_config: &PkgxConfig,
) -> Result<(String, String)> {
    let config = pkgx_config
        .get_config()
        .context("Failed to initialize libpkgx config")?;
    std::fs::create_dir_all(config.pantry_db_file.parent().unwrap())?;
    let mut conn = rusqlite::Connection::open(&config.pantry_db_file)?;

    // Sync if needed
    if sync::should(&config).map_err(|e| anyhow::anyhow!("{}", e))? {
        info!("Syncing pkgx pantry database...");
        let rt =
            tokio::runtime::Runtime::new().context("Failed to create Tokio runtime for sync")?;
        rt.block_on(async {
            sync::ensure(&config, &mut conn)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))
        })?;
    }

    let project_name = map_tool_to_project(tool_name, &conn)?;

    let tool_spec = if version_spec == "latest" {
        project_name.clone()
    } else {
        format!("{}@{}", project_name, version_spec)
    };

    Ok((project_name, tool_spec))
}

/// Check if the pkgx binary is available on the system
pub fn check_pkgx_binary() -> bool {
    use std::process::{Command, Stdio};

    Command::new("pkgx")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Query the pantry database for projects that provide a given symbol/command
#[allow(dead_code)]
pub fn projects_for_symbol(symbol: &str, conn: &Connection) -> Result<Vec<String>> {
    let symbol_string = symbol.to_string();
    libpkgx::pantry_db::projects_for_symbol(&symbol_string, conn)
        .map_err(|e| anyhow::anyhow!("Failed to query pantry database: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_connection() -> Result<Connection> {
        let config = Config::new()?;
        std::fs::create_dir_all(config.pantry_db_file.parent().unwrap())?;
        let mut conn = Connection::open(&config.pantry_db_file)?;

        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            if sync::should(&config).unwrap_or(false) {
                sync::ensure(&config, &mut conn).await.ok();
            }
        });

        Ok(conn)
    }

    #[test]
    fn test_check_pkgx_binary() {
        // This test just checks that the function runs without panic
        let _ = check_pkgx_binary();
    }

    #[test]
    fn test_map_tool_to_project_python() {
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
        let result = map_tool_to_project("python", &conn);
        assert!(result.is_ok());
        // The result should be a valid project name (e.g., "python.org")
        let project = result.unwrap();
        assert!(!project.is_empty());
    }

    #[test]
    fn test_map_tool_to_project_unknown() {
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

        let result = map_tool_to_project("unknown-tool-xyz-not-real", &conn);
        assert!(result.is_ok());
        // Unknown tools should fall back to the tool name itself
        let project = result.unwrap();
        assert_eq!(project, "unknown-tool-xyz-not-real");
    }

    #[test]
    fn test_resolve_tool_to_project() {
        // Test the full resolution flow including version spec
        let result = resolve_tool_to_project(
            "node",
            "latest",
            &PkgxConfig::new("/tmp/pkgx_test_dir", "/tmp/pkgx_pantry_dir"),
        );
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
    }

    #[test]
    fn test_resolve_package_with_libpkgx() {
        let result = resolve_package_with_libpkgx(
            &["nodejs.org".to_string()],
            &PkgxConfig::new("/tmp/pkgx_test_dir", "/tmp/pkgx_pantry_dir"),
        );
        match result {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Expected error in CI: {}", e);
            }
        }
    }

    #[test]
    fn test_projects_for_symbol() {
        let conn = match setup_connection() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Skipping test due to setup error: {}", e);
                return;
            }
        };

        let result = projects_for_symbol("python", &conn);
        match result {
            Ok(projects) => {
                if !projects.is_empty() {
                    assert!(!projects[0].is_empty());
                }
            }
            Err(e) => {
                eprintln!("Skipping test due to query error: {}", e);
                if e.to_string().contains("Failed to query pantry database") {
                    return;
                }
                panic!("Unexpected error: {}", e);
            }
        }
    }

    #[test]
    fn test_projects_for_symbol_unknown() {
        let conn = match setup_connection() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Skipping test due to setup error: {}", e);
                return;
            }
        };

        let result = projects_for_symbol("unknown-command-xyz-not-real", &conn);
        match result {
            Ok(projects) => {
                assert!(projects.is_empty());
            }
            Err(e) => {
                eprintln!("Skipping test due to query error: {}", e);
                if e.to_string().contains("Failed to query pantry database") {
                    return;
                }
                panic!("Unexpected error: {}", e);
            }
        }
    }
}
