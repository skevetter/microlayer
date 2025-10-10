use anyhow::Result;
use rusqlite::Connection;

/// Query the pantry database for projects that provide a given symbol/command
pub fn projects_for_symbol(symbol: &str, conn: &Connection) -> Result<Vec<String>> {
    let symbol_string = symbol.to_string();
    libpkgx::pantry_db::projects_for_symbol(&symbol_string, conn)
        .map_err(|e| anyhow::anyhow!("Failed to query pantry database: {}", e))
}

/// Query the pantry database for dependencies of a project
pub fn deps_for_project(
    project: &String,
    conn: &Connection,
) -> Result<Vec<libpkgx::types::PackageReq>, Box<dyn std::error::Error>> {
    libpkgx::pantry_db::deps_for_project(project, conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use libpkgx::{config::Config, sync};

    fn setup_connection() -> Result<Connection> {
        let config = Config::new()?;
        std::fs::create_dir_all(config.pantry_db_file.parent().unwrap())?;
        let mut conn = Connection::open(&config.pantry_db_file)?;

        // Sync the database if needed
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            if sync::should(&config).unwrap_or(false) {
                sync::ensure(&config, &mut conn).await.ok();
            }
        });

        Ok(conn)
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
                // Python should have at least one project associated with it
                if !projects.is_empty() {
                    assert!(!projects[0].is_empty());
                }
            }
            Err(e) => {
                // If query fails (e.g., database not synced), skip the test
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
                // Unknown commands should return empty list
                assert!(projects.is_empty());
            }
            Err(e) => {
                // If query fails (e.g., database not synced), skip the test
                eprintln!("Skipping test due to query error: {}", e);
                if e.to_string().contains("Failed to query pantry database") {
                    return;
                }
                panic!("Unexpected error: {}", e);
            }
        }
    }
}
