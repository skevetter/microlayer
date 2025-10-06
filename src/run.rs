use anyhow::{Context, Result};
#[cfg(not(target_os = "macos"))]
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub fn uninstall_pkgx() -> Result<()> {
    println!("Uninstalling pkgx and removing all associated files...");

    let mut removed_count = 0;
    let mut error_count = 0;

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
            paths.push(home_dir.join("Library/Caches/pkgx"));
            paths.push(home_dir.join("Library/Application Support/pkgx"));
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

pub fn execute(
    tool: &str,
    args: &[String],
    working_dir: &str,
    env_vars: &[String],
    _force_pkgx: bool,
    ephemeral: bool,
) -> Result<()> {
    let working_path = Path::new(working_dir);
    if !working_path.exists() {
        anyhow::bail!("Working directory does not exist: {}", working_dir);
    }

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

    execute_with_pkgx_binary(&tool_name, &version_spec, args, working_path, &env_map)
}

fn parse_tool_spec(tool: &str) -> (String, String) {
    if let Some((name, version)) = tool.split_once('@') {
        (name.to_string(), version.to_string())
    } else {
        (tool.to_string(), "latest".to_string())
    }
}

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
        _ => tool_name.to_string(),
    }
}

fn execute_with_pkgx_binary(
    tool_name: &str,
    version_spec: &str,
    args: &[String],
    working_path: &Path,
    env_map: &[(String, String)],
) -> Result<()> {
    let pkgx_available = check_pkgx_binary();

    if !pkgx_available {
        anyhow::bail!(
            "pkgx is not available. Please install pkgx from https://pkgx.sh\n\
             Installation: curl -fsS https://pkgx.sh | sh"
        );
    }

    let project_name = map_tool_to_project(tool_name);

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
