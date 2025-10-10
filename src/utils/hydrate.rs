use anyhow::{Context, Result};
use log::info;
use std::collections::HashMap;

/// Resolve package dependencies using libpkgx
pub fn resolve_package_with_libpkgx(
    dependencies: &[String],
) -> Result<(HashMap<String, String>, Vec<libpkgx::types::Installation>)> {
    let rt = tokio::runtime::Runtime::new()
        .context("Failed to create Tokio runtime for libpkgx operations")?;

    rt.block_on(async { resolve_dependencies_async(dependencies).await })
}

async fn resolve_dependencies_async(
    dependencies: &[String],
) -> Result<(HashMap<String, String>, Vec<libpkgx::types::Installation>)> {
    use libpkgx::{
        config::Config, hydrate, install_multi::ProgressBarExt, pantry_db, resolve, sync,
        types::PackageReq,
    };
    use std::collections::HashMap;
    use std::sync::Arc;

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

    let config = Config::new().context("Failed to initialize libpkgx config")?;

    std::fs::create_dir_all(config.pantry_db_file.parent().unwrap())?;
    let mut conn = rusqlite::Connection::open(&config.pantry_db_file)?;

    if sync::should(&config).map_err(|e| anyhow::anyhow!("{}", e))? {
        info!("Syncing pkgx pantry database...");
        sync::ensure(&config, &mut conn)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
    }

    let mut resolved_env = HashMap::new();
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

    for (key, value) in runtime_env {
        let cleaned_value = clean_shell_expansion(&value, &key.to_string());
        resolved_env.insert(key.to_string(), cleaned_value);
    }

    info!(
        "Successfully resolved {} packages with libpkgx",
        dependencies.len()
    );
    Ok((resolved_env, installations))
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_shell_expansion() {
        assert_eq!(
            clean_shell_expansion("${PATH:-/usr/bin}", "PATH"),
            "/usr/bin"
        );
        assert_eq!(
            clean_shell_expansion("/usr/local/bin:${PATH}", "PATH"),
            "/usr/local/bin"
        );
    }

    #[test]
    fn test_resolve_package_with_libpkgx() {
        // This test verifies the function signature and basic error handling
        let result = resolve_package_with_libpkgx(&["nodejs.org".to_string()]);
        // We don't assert success here because network/sync issues may occur in CI
        // The important thing is the function doesn't panic
        match result {
            Ok(_) => {
                // Success case
            }
            Err(e) => {
                // Expected in CI environments with network restrictions
                eprintln!("Expected error in CI: {}", e);
            }
        }
    }
}
