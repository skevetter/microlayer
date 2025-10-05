use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use regex::Regex;
use reqwest::blocking::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
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
    pattern: Option<&str>,
    verify_checksum: bool,
) -> Result<()> {
    let client = Client::new();

    // Get release information
    let release = get_release(&client, repo, version)?;

    println!("Installing from release: {}", release.tag_name);

    // Detect current platform
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;

    // Find suitable asset
    let asset = find_suitable_asset(&release.assets, arch, os, pattern)
        .context("No suitable asset found for this platform")?;

    println!("Downloading: {}", asset.name);

    // Verify checksum if requested
    if verify_checksum {
        verify_asset_checksum(&client, &release.assets, asset)?;
    }

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
        .header("User-Agent", "picolayer")
        .send()
        .context("Failed to fetch release information")?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to fetch release: {}", response.status());
    }

    let release: Release = response.json().context("Failed to parse release JSON")?;
    Ok(release)
}

fn find_suitable_asset<'a>(
    assets: &'a [Asset],
    arch: &str,
    os: &str,
    pattern: Option<&str>,
) -> Option<&'a Asset> {
    // If a regex pattern is provided, use it
    if let Some(pattern_str) = pattern {
        if let Ok(regex) = Regex::new(pattern_str) {
            return assets.iter().find(|a| regex.is_match(&a.name));
        }
    }

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
            || name_lower.ends_with(".tar.xz")
            || name_lower.ends_with(".zip");

        if has_arch && has_os && is_archive {
            return Some(asset);
        }
    }

    // Fallback: just return first archive
    assets.iter().find(|a| {
        let name = a.name.to_lowercase();
        name.ends_with(".tar.gz")
            || name.ends_with(".tgz")
            || name.ends_with(".tar.xz")
            || name.ends_with(".zip")
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
        .header("User-Agent", "picolayer")
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

fn verify_asset_checksum(client: &Client, assets: &[Asset], asset: &Asset) -> Result<()> {
    println!("Verifying checksum...");

    // Look for checksum files (common patterns: .sha256, .sha256sum, .asc, checksums.txt)
    let checksum_patterns = [
        format!("{}.sha256", asset.name),
        format!("{}.sha256sum", asset.name),
        format!("{}.asc", asset.name),
        "checksums.txt".to_string(),
        "SHA256SUMS".to_string(),
        "sha256sums.txt".to_string(),
    ];

    let checksum_asset = assets.iter().find(|a| {
        checksum_patterns
            .iter()
            .any(|pattern| a.name == *pattern || a.name.to_lowercase() == pattern.to_lowercase())
    });

    if let Some(checksum_asset) = checksum_asset {
        println!("Found checksum file: {}", checksum_asset.name);

        // Download the asset to calculate its hash
        let asset_response = client
            .get(&asset.browser_download_url)
            .header("User-Agent", "picolayer")
            .send()
            .context("Failed to download asset for checksum verification")?;

        if !asset_response.status().is_success() {
            anyhow::bail!("Failed to download asset: {}", asset_response.status());
        }

        let asset_bytes = asset_response.bytes()?;
        let mut hasher = Sha256::new();
        hasher.update(&asset_bytes);
        let computed_hash = hex::encode(hasher.finalize());

        // Download checksum file
        let checksum_response = client
            .get(&checksum_asset.browser_download_url)
            .header("User-Agent", "picolayer")
            .send()
            .context("Failed to download checksum file")?;

        if !checksum_response.status().is_success() {
            anyhow::bail!(
                "Failed to download checksum file: {}",
                checksum_response.status()
            );
        }

        let checksum_content = checksum_response.text()?;

        // Parse checksum file - handle various formats
        let expected_hash = parse_checksum_file(&checksum_content, &asset.name)?;

        if computed_hash.to_lowercase() == expected_hash.to_lowercase() {
            println!("✓ Checksum verification passed");
            Ok(())
        } else {
            anyhow::bail!(
                "Checksum verification failed!\nExpected: {}\nComputed: {}",
                expected_hash,
                computed_hash
            );
        }
    } else {
        println!("Warning: No checksum file found, skipping verification");
        Ok(())
    }
}

fn parse_checksum_file(content: &str, asset_name: &str) -> Result<String> {
    // Try to find the hash for the specific asset
    // Common format: "hash  filename" or "hash *filename" or just "hash"
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Check if this line contains our asset name
        if line.contains(asset_name) {
            // Extract the hash (first field)
            if let Some(hash) = line.split_whitespace().next() {
                return Ok(hash.to_string());
            }
        }
    }

    // If no specific line found, assume the entire content is the hash
    let hash = content.split_whitespace().next().unwrap_or("");
    if hash.len() == 64 {
        // SHA256 hash length
        Ok(hash.to_string())
    } else {
        anyhow::bail!("Could not parse checksum file for asset: {}", asset_name)
    }
}
