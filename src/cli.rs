use crate::installers;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::collections::HashMap;

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

        #[command(flatten)]
        ppa_args: PpaArgs,
    },

    /// Install packages using apt
    Apt {
        /// Comma-separated list of packages to install
        packages: String,

        #[command(flatten)]
        ppa_args: PpaArgs,
    },

    /// Install packages using aptitude
    Aptitude {
        /// Comma-separated list of packages to install
        packages: String,
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

    /// Install a devcontainer feature
    #[command(name = "devcontainer-feature")]
    DevcontainerFeature {
        /// OCI feature reference (e.g., ghcr.io/devcontainers/features/node:1)
        feature: String,

        /// Feature options (key=value pairs)
        #[arg(long)]
        option: Vec<String>,

        /// Remote user for feature installation
        #[arg(long)]
        remote_user: Option<String>,

        /// Environment variables (key=value pairs)
        #[arg(long)]
        env: Vec<String>,
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
    X {
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
    },
}

/// Common PPA arguments for apt-based installers
#[derive(clap::Args)]
struct PpaArgs {
    /// Comma-separated list of PPAs to use
    #[arg(long)]
    ppas: Option<String>,

    /// Force PPAs on non-Ubuntu systems
    #[arg(long, default_value = "false")]
    force_ppas_on_non_ubuntu: bool,
}

pub fn cli() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::AptGet { packages, ppa_args } => {
            install_apt_based(packages, ppa_args, installers::apt_get::install)?;
        }

        Commands::Apt { packages, ppa_args } => {
            install_apt_based(packages, ppa_args, installers::apt::install)?;
        }

        Commands::Aptitude { packages } => {
            let pkg_list = normalize_package_list(&packages);
            installers::aptitude::install(&pkg_list)?;
        }

        Commands::Apk { packages } => {
            let pkg_list = normalize_package_list(&packages);
            installers::apk::install(&pkg_list)?;
        }

        Commands::Brew { packages } => {
            let pkg_list = normalize_package_list(&packages);
            installers::brew::install(&pkg_list)?;
        }

        Commands::DevcontainerFeature {
            feature,
            option,
            remote_user,
            env,
        } => {
            let options = parse_key_value_pairs(&option);
            let envs = parse_key_value_pairs(&env);

            installers::devcontainer_feature::install(
                &feature,
                options,
                remote_user.as_deref(),
                envs,
            )?;
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
            let binary_list = normalize_package_list(&binary_names);

            installers::gh_release::install(&installers::gh_release::GhReleaseConfig {
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

        Commands::X {
            tool,
            args,
            working_dir,
            env,
        } => {
            installers::x::execute(&installers::x::RunConfig {
                tool: &tool,
                args,
                working_dir: &working_dir,
                env_vars: env,
            })?;
        }
    }

    Ok(())
}

/// Helper function to install apt-based packages with PPA support
fn install_apt_based<F>(packages: String, ppa_args: PpaArgs, install_fn: F) -> Result<()>
where
    F: FnOnce(&[String], Option<&[String]>, bool) -> Result<()>,
{
    let pkg_list = normalize_package_list(&packages);
    let ppa_list = ppa_args.ppas.as_ref().map(|p| normalize_package_list(p));

    install_fn(
        &pkg_list,
        ppa_list.as_deref(),
        ppa_args.force_ppas_on_non_ubuntu,
    )
}

/// Parse comma-separated string into a vector of trimmed strings
fn normalize_package_list(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Parse key=value pairs into a HashMap
fn parse_key_value_pairs(pairs: &[String]) -> Option<HashMap<String, String>> {
    if pairs.is_empty() {
        return None;
    }

    let map: HashMap<String, String> = pairs
        .iter()
        .filter_map(|pair| {
            pair.split_once('=')
                .map(|(k, v)| (k.to_string(), v.to_string()))
        })
        .collect();

    if map.is_empty() { None } else { Some(map) }
}
