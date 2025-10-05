use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::Path;
use tar::Archive;

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Debug, Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

/// Install binaries from a GitHub release
pub fn install(
    repo: &str,
    binary_names: &[String],
    version: &str,
    bin_location: &str,
) -> Result<()> {
    let client = Client::new();

    // Get release information
    let release = get_release(&client, repo, version)?;

    println!("Installing from release: {}", release.tag_name);

    // Detect current platform
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;

    // Find suitable asset
    let asset = find_suitable_asset(&release.assets, arch, os)
        .context("No suitable asset found for this platform")?;

    println!("Downloading: {}", asset.name);

    // Download and extract
    download_and_install(
        &client,
        &asset.browser_download_url,
        binary_names,
        bin_location,
    )?;

    println!("Installation complete!");
    Ok(())
}

fn get_release(client: &Client, repo: &str, version: &str) -> Result<Release> {
    let url = if version == "latest" {
        format!("https://api.github.com/repos/{}/releases/latest", repo)
    } else {
        format!(
            "https://api.github.com/repos/{}/releases/tags/{}",
            repo, version
        )
    };

    let response = client
        .get(&url)
        .header("User-Agent", "microlayer")
        .send()
        .context("Failed to fetch release information")?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to fetch release: {}", response.status());
    }

    let release: Release = response.json().context("Failed to parse release JSON")?;
    Ok(release)
}

fn find_suitable_asset<'a>(assets: &'a [Asset], arch: &str, os: &str) -> Option<&'a Asset> {
    // Map Rust arch names to common naming conventions
    let arch_patterns = match arch {
        "x86_64" => vec!["x86_64", "amd64", "x64"],
        "aarch64" => vec!["aarch64", "arm64"],
        "arm" => vec!["arm", "armv7"],
        _ => vec![arch],
    };

    // Map Rust OS names to common naming conventions
    let os_patterns = match os {
        "linux" => vec!["linux", "Linux"],
        "macos" => vec!["darwin", "macos", "osx", "Darwin"],
        _ => vec![os],
    };

    // Try to find asset matching architecture and OS
    for asset in assets {
        let name_lower = asset.name.to_lowercase();

        let has_arch = arch_patterns
            .iter()
            .any(|p| name_lower.contains(&p.to_lowercase()));
        let has_os = os_patterns
            .iter()
            .any(|p| name_lower.contains(&p.to_lowercase()));

        // Check if it's an archive format
        let is_archive = name_lower.ends_with(".tar.gz")
            || name_lower.ends_with(".tgz")
            || name_lower.ends_with(".zip");

        if has_arch && has_os && is_archive {
            return Some(asset);
        }
    }

    // Fallback: just return first archive
    assets.iter().find(|a| {
        let name = a.name.to_lowercase();
        name.ends_with(".tar.gz") || name.ends_with(".tgz") || name.ends_with(".zip")
    })
}

fn download_and_install(
    client: &Client,
    url: &str,
    binary_names: &[String],
    bin_location: &str,
) -> Result<()> {
    // Download to temp file
    let response = client
        .get(url)
        .header("User-Agent", "microlayer")
        .send()
        .context("Failed to download asset")?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to download asset: {}", response.status());
    }

    let temp_dir = tempfile::tempdir()?;
    let archive_path = temp_dir.path().join("download.tar.gz");
    let mut file = File::create(&archive_path)?;
    file.write_all(&response.bytes()?)?;

    // Extract archive
    let file = File::open(&archive_path)?;
    let reader = BufReader::new(file);
    let decoder = GzDecoder::new(reader);
    let mut archive = Archive::new(decoder);

    // Create bin location if it doesn't exist
    fs::create_dir_all(bin_location).context("Failed to create bin directory")?;

    // Extract matching binaries
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        // Check if this is one of the binaries we want
        if binary_names.iter().any(|name| name == &file_name) {
            let dest_path = Path::new(bin_location).join(&file_name);
            entry.unpack(&dest_path)?;

            // Make executable
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&dest_path)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&dest_path, perms)?;
            }

            println!("Installed: {} -> {}", file_name, dest_path.display());
        }
    }

    Ok(())
}
