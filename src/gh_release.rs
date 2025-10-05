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
    filter: Option<&str>,
    checksum: bool,
    gpg_key: Option<&str>,
) -> Result<()> {
    let client = Client::new();
    let release = get_release(&client, repo, version)?;

    println!("Installing from release: {}", release.tag_name);

    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;
    let asset = find_suitable_asset(&release.assets, arch, os, filter)
        .context("No suitable asset found for this platform")?;

    println!("Downloading: {}", asset.name);

    if checksum {
        verify_asset_checksum(&client, &release.assets, asset, gpg_key)?;
    }

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
    filter: Option<&str>,
) -> Option<&'a Asset> {
    if let Some(pattern_str) = filter {
        if let Ok(regex) = Regex::new(pattern_str) {
            return assets.iter().find(|a| regex.is_match(&a.name));
        }
    }

    let arch_patterns = match arch {
        "x86_64" => vec!["x86_64", "amd64", "x64", "x86-64"],
        "aarch64" => vec!["aarch64", "arm64"],
        "arm" => vec!["arm", "armv7"],
        _ => vec![arch],
    };
    let os_patterns = match os {
        "linux" => vec!["linux", "Linux"],
        "macos" => vec!["darwin", "macos", "osx", "Darwin"],
        _ => vec![os],
    };

    for asset in assets {
        let name_lower = asset.name.to_lowercase();

        let has_arch = arch_patterns
            .iter()
            .any(|p| name_lower.contains(&p.to_lowercase()));
        let has_os = os_patterns
            .iter()
            .any(|p| name_lower.contains(&p.to_lowercase()));

        let is_archive = name_lower.ends_with(".tar.gz")
            || name_lower.ends_with(".tgz")
            || name_lower.ends_with(".tar.xz")
            || name_lower.ends_with(".zip");

        if has_arch && has_os && is_archive {
            return Some(asset);
        }
    }

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

    let file = File::open(&archive_path)?;
    let reader = BufReader::new(file);
    let decoder = GzDecoder::new(reader);
    let mut archive = Archive::new(decoder);

    fs::create_dir_all(bin_location).context("Failed to create bin directory")?;

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        if binary_names.iter().any(|name| name == &file_name) {
            let dest_path = Path::new(bin_location).join(&file_name);
            entry.unpack(&dest_path)?;

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

fn get_asset_variants(filename: &str) -> Vec<String> {
    let compression_extensions = [
        ".tar.gz",
        ".tgz",
        ".tar.xz",
        ".txz",
        ".tar.bz2",
        ".tbz2",
        ".tar.Z",
        ".tar.lz",
        ".tar.lzma",
        ".zip",
        ".gz",
        ".xz",
        ".bz2",
        ".Z",
        ".lz",
        ".lzma",
    ];
    let mut variants = vec![filename.to_string()];

    let mut base_name = filename;
    for ext in &compression_extensions {
        if filename.ends_with(ext) {
            base_name = filename.strip_suffix(ext).unwrap();
            break;
        }
    }

    if base_name != filename {
        variants.push(base_name.to_string());
    }

    for ext in &compression_extensions {
        let variant = format!("{}{}", base_name, ext);
        if variant != filename {
            variants.push(variant);
        }
    }

    variants
}

fn verify_asset_checksum(client: &Client, assets: &[Asset], asset: &Asset, gpg_key: Option<&str>) -> Result<()> {
    println!("Verifying checksum...");
    let asset_variants = get_asset_variants(&asset.name);
    let mut checksum_patterns = Vec::new();

    for variant in &asset_variants {
        checksum_patterns.extend([
            format!("{}.sha256", variant),
            format!("{}.sha256sum", variant),
            format!("{}.asc", variant),
            format!("{}.sig", variant),
            format!("{}.md5", variant),
            format!("{}.sha1", variant),
            format!("{}.sha512", variant),
        ]);
    }

    checksum_patterns.extend([
        "checksums.txt".to_string(),
        "SHA256SUMS".to_string(),
        "sha256sums.txt".to_string(),
        "CHECKSUMS".to_string(),
        "checksums.sha256".to_string(),
    ]);

    println!(
        "Looking for checksum files matching: {:?}",
        checksum_patterns.iter().take(10).collect::<Vec<_>>()
    );
    println!(
        "Available assets: {:?}",
        assets.iter().map(|a| &a.name).collect::<Vec<_>>()
    );

    let checksum_asset = assets.iter().find(|a| {
        checksum_patterns
            .iter()
            .any(|pattern| a.name == *pattern || a.name.to_lowercase() == pattern.to_lowercase())
    });

    if let Some(checksum_asset) = checksum_asset {
        println!("Found checksum file: {}", checksum_asset.name);
        if checksum_asset.name.ends_with(".asc") || checksum_asset.name.ends_with(".sig") {
            if let Some(key_content) = gpg_key {
                println!("Verifying GPG signature...");
                return verify_gpg_signature(client, asset, checksum_asset, key_content);
            } else {
                println!("Warning: Found signature file but no GPG key provided");
                println!("Use --gpg-key option to enable GPG verification");
                println!("Skipping checksum verification");
                return Ok(());
            }
        }

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
        let mut expected_hash = None;
        for variant in &asset_variants {
            if let Ok(hash) = parse_checksum_file(&checksum_content, variant) {
                expected_hash = Some(hash);
                break;
            }
        }

        let expected_hash =
            expected_hash.context("Could not find matching checksum in checksum file")?;

        if computed_hash.to_lowercase() == expected_hash.to_lowercase() {
            println!("Checksum verification passed");
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
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.contains(asset_name) {
            if let Some(hash) = line.split_whitespace().next() {
                return Ok(hash.to_string());
            }
        }
    }

    let hash = content.split_whitespace().next().unwrap_or("");
    if hash.len() == 64 {
        Ok(hash.to_string())
    } else {
        anyhow::bail!("Could not parse checksum file for asset: {}", asset_name)
    }
}

fn verify_gpg_signature(
    client: &Client,
    asset: &Asset,
    signature_asset: &Asset,
    gpg_key_content: &str,
) -> Result<()> {
    use pgp::composed::{Deserializable, DetachedSignature, SignedPublicKey};
    use std::io::Cursor;

    println!("Downloading asset for verification...");
    let asset_response = client
        .get(&asset.browser_download_url)
        .header("User-Agent", "picolayer")
        .send()
        .context("Failed to download asset")?;

    if !asset_response.status().is_success() {
        anyhow::bail!("Failed to download asset: {}", asset_response.status());
    }

    let asset_data = asset_response.bytes()?;

    println!("Downloading signature file...");
    let sig_response = client
        .get(&signature_asset.browser_download_url)
        .header("User-Agent", "picolayer")
        .send()
        .context("Failed to download signature")?;

    if !sig_response.status().is_success() {
        anyhow::bail!(
            "Failed to download signature: {}",
            sig_response.status()
        );
    }

    let sig_data = sig_response.bytes()?;

    // Parse the GPG key
    println!("Loading GPG public key...");
    let key_data = if std::path::Path::new(gpg_key_content).exists() {
        std::fs::read_to_string(gpg_key_content)
            .context("Failed to read GPG key file")?
    } else {
        gpg_key_content.to_string()
    };

    let (public_key, _headers) = SignedPublicKey::from_string(&key_data)
        .context("Failed to parse GPG public key")?;

    // Parse the signature
    println!("Parsing signature...");
    let signature = DetachedSignature::from_bytes(Cursor::new(&sig_data[..]))
        .context("Failed to parse signature")?;

    // Verify the signature
    println!("Verifying signature...");
    signature
        .verify(&public_key, &asset_data[..])
        .context("GPG signature verification failed")?;

    println!("GPG signature verification passed!");
    Ok(())
}

