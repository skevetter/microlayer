mod apk;
mod apt_get;
mod brew;
mod gh_release;
mod run;
mod utils;

use anyhow::Result;
use clap::{Parser, Subcommand};
use env_logger::Env;
use log::info;

#[derive(Parser)]
#[command(name = "picolayer")]
#[command(about = "Ensures minimal container layers")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install packages using apt-get
    #[command(name = "apt-get")]
    AptGet {
        /// Comma-separated list of packages to install
        packages: String,

        /// Comma-separated list of PPAs to use
        #[arg(long)]
        ppas: Option<String>,

        /// Force PPAs on non-Ubuntu systems
        #[arg(long, default_value = "false")]
        force_ppas_on_non_ubuntu: bool,
    },

    /// Install packages using apk
    Apk {
        /// Comma-separated list of packages to install
        packages: String,
    },

    /// Install packages using Homebrew
    Brew {
        /// Comma-separated list of packages to install
        packages: String,
    },

    /// Install binary from GitHub release
    #[command(name = "gh-release")]
    GhRelease {
        /// Repository (e.g., cli/cli)
        repo: String,

        /// Comma-separated list of binary names
        binary_names: String,

        /// Version to install (default: latest)
        #[arg(long, default_value = "latest")]
        version: String,

        /// Directory to install binaries
        #[arg(long, default_value = "/usr/local/bin")]
        install_dir: String,

        /// Regex pattern for asset filtering
        #[arg(long)]
        filter: Option<String>,

        /// Verify checksums using checksum files
        #[arg(long, default_value = "false", conflicts_with = "checksum_text")]
        verify_checksum: bool,

        /// Checksum text for verification (e.g., "sha256:5d3d3c60ffcf601f964bb4060a4234f9a96a3b09a7cdf67d1e61ae88efcd48f4")
        #[arg(long, conflicts_with = "verify_checksum")]
        checksum_text: Option<String>,

        /// GPG public key for signature verification (can be a URL, file path, or key content)
        #[arg(long)]
        gpg_key: Option<String>,
    },

    /// Run a command using pkgx
    Run {
        /// Tool specification (e.g., "python@3.10", "node@18", "python")
        tool: String,

        /// Arguments to pass to the tool
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,

        /// Working directory for execution
        #[arg(long, default_value = ".")]
        working_dir: String,

        /// Environment variables (key=value pairs)
        #[arg(long)]
        env: Vec<String>,

        /// Keep packages after command execution (default: delete after execution)
        #[arg(long, default_value = "false", conflicts_with = "keep_pkgx")]
        keep_package: bool,

        /// Completely uninstall pkgx and remove all cache/data files
        #[arg(long, default_value = "false", conflicts_with = "keep_package")]
        keep_pkgx: bool,
    },
}

fn normalize_pkg_input(packages: String) -> Vec<String> {
    packages.split(',').map(|s| s.trim().to_string()).collect()
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();
    info!("Starting picolayer");

    let cli = Cli::parse();

    match cli.command {
        Commands::AptGet {
            packages,
            ppas,
            force_ppas_on_non_ubuntu,
        } => {
            let pkg_list: Vec<String> = normalize_pkg_input(packages);
            let ppa_list: Option<Vec<String>> = ppas.map(normalize_pkg_input);
            
            // Track analytics
            let _ = utils::analytics::track_command("apt-get", Some(serde_json::json!({
                "package_count": pkg_list.len(),
                "has_ppas": ppa_list.is_some(),
            })));
            
            apt_get::install(&pkg_list, ppa_list.as_deref(), force_ppas_on_non_ubuntu)?;
        }

        Commands::Apk { packages } => {
            let pkg_list: Vec<String> = normalize_pkg_input(packages);
            
            // Track analytics
            let _ = utils::analytics::track_command("apk", Some(serde_json::json!({
                "package_count": pkg_list.len(),
            })));
            
            apk::install(&pkg_list)?;
        }

        Commands::Brew { packages } => {
            let pkg_list: Vec<String> = normalize_pkg_input(packages);
            
            // Track analytics
            let _ = utils::analytics::track_command("brew", Some(serde_json::json!({
                "package_count": pkg_list.len(),
            })));
            
            brew::install(&pkg_list)?;
        }

        Commands::GhRelease {
            repo,
            binary_names,
            version,
            install_dir,
            filter,
            verify_checksum,
            checksum_text,
            gpg_key,
        } => {
            let binary_list: Vec<String> = binary_names
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            
            // Track analytics
            let _ = utils::analytics::track_command("gh-release", Some(serde_json::json!({
                "repo": repo,
                "binary_count": binary_list.len(),
                "version": version,
                "has_filter": filter.is_some(),
                "verify_checksum": verify_checksum,
                "has_gpg_key": gpg_key.is_some(),
            })));
            
            gh_release::install(&gh_release::GhReleaseConfig {
                repo: &repo,
                binary_names: &binary_list,
                version: &version,
                install_dir: &install_dir,
                filter: filter.as_deref(),
                verify_checksum,
                checksum_text: checksum_text.as_deref(),
                gpg_key: gpg_key.as_deref(),
            })?;
        }

        Commands::Run {
            tool,
            args,
            working_dir,
            env,
            keep_package,
            keep_pkgx,
        } => {
            // Track analytics
            let _ = utils::analytics::track_command("run", Some(serde_json::json!({
                "tool": tool,
                "arg_count": args.len(),
                "env_count": env.len(),
                "keep_package": keep_package,
                "keep_pkgx": keep_pkgx,
            })));
            
            run::execute(&run::RunConfig {
                tool: &tool,
                args,
                working_dir: &working_dir,
                env_vars: env,
                keep_package,
                keep_pkgx,
            })?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_pkg_input_single() {
        let result = normalize_pkg_input("package1".to_string());
        assert_eq!(result, vec!["package1".to_string()]);
    }

    #[test]
    fn test_normalize_pkg_input_multiple() {
        let result = normalize_pkg_input("package1,package2,package3".to_string());
        assert_eq!(
            result,
            vec![
                "package1".to_string(),
                "package2".to_string(),
                "package3".to_string()
            ]
        );
    }

    #[test]
    fn test_normalize_pkg_input_with_spaces() {
        let result = normalize_pkg_input("package1 , package2 , package3".to_string());
        assert_eq!(
            result,
            vec![
                "package1".to_string(),
                "package2".to_string(),
                "package3".to_string()
            ]
        );
    }

    #[test]
    fn test_normalize_pkg_input_empty() {
        let result = normalize_pkg_input("".to_string());
        assert_eq!(result, vec!["".to_string()]);
    }

    #[test]
    fn test_cli_parser_exists() {
        use clap::CommandFactory;
        let _ = Cli::command();
    }

    #[test]
    fn test_commands_enum_variants() {
        use clap::CommandFactory;
        let cmd = Cli::command();
        let subcommands: Vec<_> = cmd.get_subcommands().map(|s| s.get_name()).collect();

        assert!(subcommands.contains(&"apt-get"));
        assert!(subcommands.contains(&"apk"));
        assert!(subcommands.contains(&"brew"));
        assert!(subcommands.contains(&"gh-release"));
        assert!(subcommands.contains(&"run"));
    }
}
