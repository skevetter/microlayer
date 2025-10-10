use anyhow::{Context, Result};
use log::{info, warn};

/// Query the pkgx pantry database to resolve a tool name to a project name
pub fn map_tool_to_project(tool_name: &str, conn: &rusqlite::Connection) -> Result<String> {
    // Query pkgx pantry database to resolve tool to project
    let tool_name_string = tool_name.to_string();
    match libpkgx::pantry_db::projects_for_symbol(&tool_name_string, conn) {
        Ok(projects) if !projects.is_empty() => {
            if projects.len() == 1 {
                Ok(projects[0].clone())
            } else {
                // Multiple projects provide this tool, use the first one
                // In the future, we might want to add logic to choose the best one
                info!(
                    "Multiple projects provide '{}': {:?}, using {}",
                    tool_name, projects, projects[0]
                );
                Ok(projects[0].clone())
            }
        }
        Ok(_) => {
            // No project found in pantry_db, fall back to tool name as project
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
pub fn resolve_tool_to_project(tool_name: &str, version_spec: &str) -> Result<(String, String)> {
    use libpkgx::{config::Config, sync};

    let config = Config::new().context("Failed to initialize libpkgx config")?;
    std::fs::create_dir_all(config.pantry_db_file.parent().unwrap())?;
    let mut conn = rusqlite::Connection::open(&config.pantry_db_file)?;

    // Sync if needed
    if sync::should(&config).map_err(|e| anyhow::anyhow!("{}", e))? {
        info!("Syncing pkgx pantry database...");
        let rt = tokio::runtime::Runtime::new().context("Failed to create Tokio runtime for sync")?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_pkgx_binary() {
        // This test just checks that the function runs without panic
        let _ = check_pkgx_binary();
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
        let result = map_tool_to_project("python", &conn);
        assert!(result.is_ok());
        // The result should be a valid project name (e.g., "python.org")
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

        let result = map_tool_to_project("unknown-tool-xyz-not-real", &conn);
        assert!(result.is_ok());
        // Unknown tools should fall back to the tool name itself
        let project = result.unwrap();
        assert_eq!(project, "unknown-tool-xyz-not-real");
    }

    #[test]
    fn test_resolve_tool_to_project() {
        // Test the full resolution flow including version spec
        let result = resolve_tool_to_project("node", "latest");
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
}
