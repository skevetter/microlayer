mod installers;
mod utils;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "picolayer")]
#[command(about = "Ensures minimal container layers", long_about = None)]
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

        /// Location to install binaries
        #[arg(long, default_value = "/usr/local/bin")]
        bin_location: String,

        /// Regex pattern for asset filtering
        #[arg(long)]
        pattern: Option<String>,

        /// Verify checksums using checksum files
        #[arg(long, default_value = "false")]
        verify_checksum: bool,
    },

    /// Run a command using pkgx for dependency management
    Run {
        /// Command to run (e.g., "python script.py", "node app.js")
        command: String,

        /// Working directory for execution
        #[arg(long, default_value = ".")]
        working_dir: String,

        /// Environment variables (key=value pairs)
        #[arg(long)]
        env: Vec<String>,

        /// Force pkgx even if dependencies exist locally
        #[arg(long, default_value = "false")]
        force_pkgx: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::AptGet {
            packages,
            ppas,
            force_ppas_on_non_ubuntu,
        } => {
            let pkg_list: Vec<String> = packages.split(',').map(|s| s.trim().to_string()).collect();

            let ppa_list: Option<Vec<String>> =
                ppas.map(|p| p.split(',').map(|s| s.trim().to_string()).collect());

            installers::apt_get::install(&pkg_list, ppa_list.as_deref(), force_ppas_on_non_ubuntu)?;
        }

        Commands::Apk { packages } => {
            let pkg_list: Vec<String> = packages.split(',').map(|s| s.trim().to_string()).collect();

            installers::apk::install(&pkg_list)?;
        }

        Commands::GhRelease {
            repo,
            binary_names,
            version,
            bin_location,
            pattern,
            verify_checksum,
        } => {
            let binary_list: Vec<String> = binary_names
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();

            installers::gh_release::install(
                &repo,
                &binary_list,
                &version,
                &bin_location,
                pattern.as_deref(),
                verify_checksum,
            )?;
        }

        Commands::Run {
            command,
            working_dir,
            env,
            force_pkgx,
        } => {
            installers::run::execute(&command, &working_dir, &env, force_pkgx)?;
        }
    }

    Ok(())
}
