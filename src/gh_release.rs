use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use log::{info, warn};
use regex::Regex;
use reqwest::blocking::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs::{self, File};

use std::io::{BufReader, Write};
use std::path::Path;

use tar::Archive;
use xz::read::XzDecoder;

const GITHUB_API: &str = "api.github.com";

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

/// Configuration for installing a GitHub release
struct InstallConfig<'a> {
    repo: &'a str,
    binary_names: &'a [String],
    version: &'a str,
    install_dir: &'a str,
    filter: Option<&'a str>,
    verify_checksum: bool,
    checksum_text: Option<&'a str>,
    gpg_key: Option<&'a str>,
}

/// Install binaries from a GitHub release
pub fn install(
    repo: &str,
    binary_names: &[String],
    version: &str,
    install_dir: &str,
    filter: Option<&str>,
    verify_checksum: bool,
    checksum_text: Option<&str>,
    gpg_key: Option<&str>,
) -> Result<()> {
    let config = InstallConfig {
        repo,
        binary_names,
        version,
        install_dir,
        filter,
        verify_checksum,
        checksum_text,
        gpg_key,
    };

    Installer::new().install(config)
}

struct Installer {
    client: Client,
}

impl Installer {
    fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    fn install(&self, config: InstallConfig) -> Result<()> {
        info!("Fetching release information for {}", config.repo);
        let release = self.fetch_release(config.repo, config.version)?;
        info!("Installing from release: {}", release.tag_name);

        let gpg_verification = config.verify_checksum && config.gpg_key.is_some();
        let asset = self.select_asset(&release.assets, config.filter, gpg_verification)?;
        info!("Selected asset: {}", asset.name);

        if let Some(checksum_text) = config.checksum_text {
            self.verify_asset_with_checksum_text(asset, checksum_text)?;
        } else if config.verify_checksum {
            self.verify_asset(&release.assets, asset, config.gpg_key)?;
        }

        self.download_and_install_asset(asset, config.binary_names, config.install_dir)?;

        info!("Installation complete!");
        Ok(())
    }

    fn fetch_release(&self, repo: &str, version: &str) -> Result<Release> {
        ReleaseClient::new(&self.client).fetch(repo, version)
    }

    fn select_asset<'a>(
        &self,
        assets: &'a [Asset],
        filter: Option<&str>,
        gpg_verification: bool,
    ) -> Result<&'a Asset> {
        let selector = AssetSelector::new();
        if gpg_verification {
            selector.select_with_signature(assets, filter)
        } else {
            selector.select(assets, filter)
        }
    }

    fn verify_asset(&self, assets: &[Asset], asset: &Asset, gpg_key: Option<&str>) -> Result<()> {
        AssetVerifier::new(&self.client).verify(assets, asset, gpg_key)
    }

    fn verify_asset_with_checksum_text(&self, asset: &Asset, checksum_text: &str) -> Result<()> {
        AssetVerifier::new(&self.client).verify_with_checksum_text(asset, checksum_text)
    }

    fn download_and_install_asset(
        &self,
        asset: &Asset,
        binary_names: &[String],
        bin_location: &str,
    ) -> Result<()> {
        AssetInstaller::new(&self.client).install(asset, binary_names, bin_location)
    }
}

struct ReleaseClient<'a> {
    client: &'a Client,
}

impl<'a> ReleaseClient<'a> {
    fn new(client: &'a Client) -> Self {
        Self { client }
    }

    fn fetch(&self, repo: &str, version: &str) -> Result<Release> {
        let url = self.build_url(repo, version);

        let response = self
            .client
            .get(&url)
            .header("User-Agent", "picolayer")
            .send()
            .context("Failed to fetch release information")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch release: {}", response.status());
        }

        response.json().context("Failed to parse release JSON")
    }

    fn build_url(&self, repo: &str, version: &str) -> String {
        if version == "latest" {
            format!("https://{}/repos/{}/releases/latest", GITHUB_API, repo)
        } else {
            format!(
                "https://{}/repos/{}/releases/tags/{}",
                GITHUB_API, repo, version
            )
        }
    }
}

struct AssetSelector;

impl AssetSelector {
    fn new() -> Self {
        Self
    }

    fn select<'a>(&self, assets: &'a [Asset], filter: Option<&str>) -> Result<&'a Asset> {
        if let Some(pattern) = filter {
            return self.select_by_filter(assets, pattern);
        }

        self.select_by_platform(assets)
            .or_else(|| self.select_any_archive(assets))
            .context("No suitable asset found for this platform")
    }

    fn select_with_signature<'a>(
        &self,
        assets: &'a [Asset],
        filter: Option<&str>,
    ) -> Result<&'a Asset> {
        if let Some(pattern) = filter {
            return self.select_by_filter(assets, pattern);
        }

        // First try to find a platform-specific asset that has a signature
        if let Some(asset) = self.select_by_platform_with_signature(assets) {
            return Ok(asset);
        }

        self.select_by_platform(assets)
            .or_else(|| self.select_any_archive(assets))
            .context("No suitable asset found for this platform")
    }

    fn select_by_platform_with_signature<'a>(&self, assets: &'a [Asset]) -> Option<&'a Asset> {
        let arch = std::env::consts::ARCH;
        let os = std::env::consts::OS;

        let arch_patterns = self.get_arch_patterns(arch);
        let os_patterns = self.get_os_patterns(os);

        assets.iter().find(|asset| {
            let name_lower = asset.name.to_lowercase();
            let has_arch = arch_patterns
                .iter()
                .any(|p| name_lower.contains(&p.to_lowercase()));
            let has_os = os_patterns
                .iter()
                .any(|p| name_lower.contains(&p.to_lowercase()));
            let is_archive = self.is_archive(&name_lower);

            // Check for a signature file
            let has_signature = assets.iter().any(|sig_asset| {
                sig_asset.name == format!("{}.asc", asset.name)
                    || sig_asset.name == format!("{}.sig", asset.name)
            });

            has_arch && has_os && is_archive && has_signature
        })
    }

    fn select_by_filter<'a>(&self, assets: &'a [Asset], pattern: &str) -> Result<&'a Asset> {
        let regex = Regex::new(pattern).context("Invalid filter pattern")?;
        assets
            .iter()
            .find(|a| regex.is_match(&a.name))
            .context("No asset matching filter pattern")
    }

    fn select_by_platform<'a>(&self, assets: &'a [Asset]) -> Option<&'a Asset> {
        let arch = std::env::consts::ARCH;
        let os = std::env::consts::OS;

        let arch_patterns = self.get_arch_patterns(arch);
        let os_patterns = self.get_os_patterns(os);

        assets.iter().find(|asset| {
            let name_lower = asset.name.to_lowercase();
            let has_arch = arch_patterns
                .iter()
                .any(|p| name_lower.contains(&p.to_lowercase()));
            let has_os = os_patterns
                .iter()
                .any(|p| name_lower.contains(&p.to_lowercase()));
            let is_archive = self.is_archive(&name_lower);

            has_arch && has_os && is_archive
        })
    }

    fn select_any_archive<'a>(&self, assets: &'a [Asset]) -> Option<&'a Asset> {
        assets
            .iter()
            .find(|a| self.is_archive(&a.name.to_lowercase()))
    }

    fn get_arch_patterns(&self, arch: &str) -> Vec<&'static str> {
        match arch {
            "x86_64" => vec!["x86_64", "amd64", "x64", "x86-64"],
            "aarch64" => vec!["aarch64", "arm64"],
            "arm" => vec!["arm", "armv7"],
            _ => vec![],
        }
    }

    fn get_os_patterns(&self, os: &str) -> Vec<&'static str> {
        match os {
            "linux" => vec!["linux", "Linux"],
            "macos" => vec!["darwin", "macos", "osx", "Darwin"],
            _ => vec![],
        }
    }

    fn is_archive(&self, filename: &str) -> bool {
        filename.ends_with(".tar.gz")
            || filename.ends_with(".tgz")
            || filename.ends_with(".tar.xz")
            || filename.ends_with(".zip")
    }
}

pub struct AssetInstaller<'a> {
    client: &'a Client,
}

impl<'a> AssetInstaller<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    fn install(&self, asset: &Asset, binary_names: &[String], bin_location: &str) -> Result<()> {
        info!("Downloading asset...");
        let archive_data = self.download_asset(asset)?;

        info!("Extracting binaries: {}", binary_names.join(", "));
        self.extract_binaries(&archive_data, binary_names, bin_location)?;

        Ok(())
    }

    fn download_asset(&self, asset: &Asset) -> Result<Vec<u8>> {
        let response = self
            .client
            .get(&asset.browser_download_url)
            .header("User-Agent", "picolayer")
            .send()
            .context("Failed to download asset")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to download asset: {}", response.status());
        }

        Ok(response.bytes()?.to_vec())
    }

    fn extract_binaries(
        &self,
        archive_data: &[u8],
        binary_names: &[String],
        bin_location: &str,
    ) -> Result<()> {
        let temp_dir = tempfile::tempdir()?;

        if self.is_tar_xz_archive(archive_data) {
            self.extract_tar_xz(archive_data, binary_names, bin_location, &temp_dir)
        } else {
            self.extract_tar_gz(archive_data, binary_names, bin_location, &temp_dir)
        }
    }

    pub fn is_tar_xz_archive(&self, data: &[u8]) -> bool {
        // XZ files start with 0xFD, '7', 'z', 'X', 'Z', 0x00
        data.len() >= 6 && data[0] == 0xFD && &data[1..6] == b"7zXZ\x00"
    }

    fn extract_tar_gz(
        &self,
        archive_data: &[u8],
        binary_names: &[String],
        bin_location: &str,
        temp_dir: &tempfile::TempDir,
    ) -> Result<()> {
        let archive_path = temp_dir.path().join("download.tar.gz");

        let mut file = File::create(&archive_path)?;
        file.write_all(archive_data)?;

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
                self.install_binary(&mut entry, &file_name, bin_location)?;
            }
        }

        Ok(())
    }

    pub fn extract_tar_xz(
        &self,
        archive_data: &[u8],
        binary_names: &[String],
        bin_location: &str,
        temp_dir: &tempfile::TempDir,
    ) -> Result<()> {
        let extract_dir = temp_dir.path().join("extracted");

        fs::create_dir_all(&extract_dir)?;
        fs::create_dir_all(bin_location).context("Failed to create bin directory")?;

        let cursor = std::io::Cursor::new(archive_data);
        let xz_decoder = XzDecoder::new(cursor);
        let mut archive = Archive::new(xz_decoder);

        archive
            .unpack(&extract_dir)
            .context("Failed to extract tar.xz archive")?;

        self.find_and_install_binaries(&extract_dir, binary_names, bin_location)?;

        Ok(())
    }

    fn find_and_install_binaries(
        &self,
        extract_dir: &std::path::Path,
        binary_names: &[String],
        bin_location: &str,
    ) -> Result<()> {
        for entry in walkdir::WalkDir::new(extract_dir) {
            let entry = entry?;
            if entry.file_type().is_file() {
                let file_name = entry.file_name().to_str().unwrap_or("").to_string();

                if binary_names.iter().any(|name| name == &file_name) {
                    let source_path = entry.path();
                    let dest_path = std::path::Path::new(bin_location).join(&file_name);

                    fs::copy(source_path, &dest_path)
                        .with_context(|| format!("Failed to copy binary: {}", file_name))?;

                    // Make the binary executable
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let mut perms = fs::metadata(&dest_path)?.permissions();
                        perms.set_mode(0o755);
                        fs::set_permissions(&dest_path, perms)?;
                    }

                    info!("Installed: {} -> {}", file_name, dest_path.display());
                }
            }
        }

        Ok(())
    }

    fn install_binary(
        &self,
        entry: &mut tar::Entry<impl std::io::Read>,
        file_name: &str,
        bin_location: &str,
    ) -> Result<()> {
        let dest_path = Path::new(bin_location).join(file_name);
        entry.unpack(&dest_path)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&dest_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&dest_path, perms)?;
        }

        info!("Installed: {} -> {}", file_name, dest_path.display());
        Ok(())
    }
}

struct AssetVerifier<'a> {
    client: &'a Client,
}

impl<'a> AssetVerifier<'a> {
    fn new(client: &'a Client) -> Self {
        Self { client }
    }

    fn verify(&self, assets: &[Asset], asset: &Asset, gpg_key: Option<&str>) -> Result<()> {
        info!("Verifying asset...");

        let checksum_asset = self.find_checksum_asset(assets, asset)?;

        if checksum_asset.name.ends_with(".asc") || checksum_asset.name.ends_with(".sig") {
            return self.verify_gpg_signature(asset, checksum_asset, gpg_key);
        }

        self.verify_sha256_checksum(asset, checksum_asset, assets)
    }

    fn verify_with_checksum_text(&self, asset: &Asset, checksum_text: &str) -> Result<()> {
        info!("Verifying asset with provided checksum text...");

        // Parse checksum text format: "algorithm:hash"
        let parts: Vec<&str> = checksum_text.splitn(2, ':').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid checksum text format. Expected 'algorithm:hash' (e.g., 'sha256:abc123...')");
        }

        let algorithm = parts[0].to_lowercase();
        let expected_hash = parts[1];

        if algorithm != "sha256" {
            anyhow::bail!("Only sha256 algorithm is currently supported");
        }

        info!("Downloading asset for verification...");
        let asset_data = self.download_asset(asset)?;
        let computed_hash = compute_sha256(&asset_data);

        if computed_hash.to_lowercase() == expected_hash.to_lowercase() {
            info!("Checksum verification passed");
            Ok(())
        } else {
            anyhow::bail!(
                "Checksum verification failed!\nExpected: {}\nComputed: {}",
                expected_hash,
                computed_hash
            );
        }
    }

    fn find_checksum_asset<'b>(&self, assets: &'b [Asset], asset: &Asset) -> Result<&'b Asset> {
        // First, try to find exact signature matches for the asset
        let exact_sig_patterns = [format!("{}.asc", asset.name), format!("{}.sig", asset.name)];

        // Look for exact signature match first
        if let Some(exact_match) = assets.iter().find(|a| exact_sig_patterns.contains(&a.name)) {
            return Ok(exact_match);
        }

        // If no exact signature match, fall back to the existing logic
        let patterns = self.build_checksum_patterns(&asset.name);

        assets
            .iter()
            .find(|a| {
                patterns.iter().any(|pattern| {
                    a.name == *pattern || a.name.to_lowercase() == pattern.to_lowercase()
                })
            })
            .context("No checksum file found")
    }

    fn build_checksum_patterns(&self, filename: &str) -> Vec<String> {
        let mut patterns = Vec::new();
        let variants = get_filename_variants(filename);

        for variant in &variants {
            patterns.extend([
                format!("{}.sha256", variant),
                format!("{}.sha256sum", variant),
                format!("{}.asc", variant),
                format!("{}.sig", variant),
            ]);
        }

        patterns.extend([
            "checksums.txt".to_string(),
            "SHA256SUMS".to_string(),
            "sha256sums.txt".to_string(),
            "CHECKSUMS".to_string(),
            "checksums.sha256".to_string(),
        ]);

        patterns
    }

    fn verify_gpg_signature(
        &self,
        asset: &Asset,
        signature_asset: &Asset,
        gpg_key: Option<&str>,
    ) -> Result<()> {
        if let Some(key_content) = gpg_key {
            info!("Verifying GPG signature...");
            GpgVerifier::new(self.client).verify(asset, signature_asset, key_content)
        } else {
            warn!("Found signature file but no GPG key provided");
            info!("Use --gpg-key option to enable GPG verification");
            Ok(())
        }
    }

    fn verify_sha256_checksum(
        &self,
        asset: &Asset,
        checksum_asset: &Asset,
        _assets: &[Asset],
    ) -> Result<()> {
        info!("Verifying SHA256 checksum...");
        info!("Checksum file: {}", checksum_asset.name);

        let asset_data = self.download_asset(asset)?;
        let computed_hash = compute_sha256(&asset_data);

        let checksum_content = self.download_checksum(checksum_asset)?;
        let expected_hash = self.parse_checksum(&checksum_content, &asset.name)?;

        if computed_hash.to_lowercase() == expected_hash.to_lowercase() {
            info!("Checksum verification passed");
            Ok(())
        } else {
            anyhow::bail!(
                "Checksum verification failed!\nExpected: {}\nComputed: {}",
                expected_hash,
                computed_hash
            );
        }
    }

    fn download_asset(&self, asset: &Asset) -> Result<Vec<u8>> {
        let response = self
            .client
            .get(&asset.browser_download_url)
            .header("User-Agent", "picolayer")
            .send()
            .context("Failed to download asset")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to download asset: {}", response.status());
        }

        Ok(response.bytes()?.to_vec())
    }

    fn download_checksum(&self, checksum_asset: &Asset) -> Result<String> {
        let response = self
            .client
            .get(&checksum_asset.browser_download_url)
            .header("User-Agent", "picolayer")
            .send()
            .context("Failed to download checksum")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to download checksum: {}", response.status());
        }

        Ok(response.text()?)
    }

    fn parse_checksum(&self, content: &str, asset_name: &str) -> Result<String> {
        let variants = get_filename_variants(asset_name);

        for variant in &variants {
            if let Ok(hash) = parse_checksum_line(content, variant) {
                return Ok(hash);
            }
        }

        // Fallback: try to get the first hash-like string
        let hash = content.split_whitespace().next().unwrap_or("");
        if hash.len() == 64 {
            Ok(hash.to_string())
        } else {
            anyhow::bail!("Could not parse checksum for asset: {}", asset_name)
        }
    }
}

struct GpgVerifier<'a> {
    client: &'a Client,
}

impl<'a> GpgVerifier<'a> {
    fn new(client: &'a Client) -> Self {
        Self { client }
    }

    fn verify(&self, asset: &Asset, signature_asset: &Asset, gpg_key_content: &str) -> Result<()> {
        use pgp::composed::{Deserializable, DetachedSignature};
        use std::io::Cursor;

        info!("Downloading asset for verification...");
        let asset_data = self.download_data(&asset.browser_download_url)?;

        info!("Downloading signature file...");
        let sig_data = self.download_data(&signature_asset.browser_download_url)?;

        info!("Loading GPG public key...");
        let public_key = self.load_public_key(gpg_key_content)?;

        info!("Parsing signature...");
        let signature = if sig_data.starts_with(b"-----BEGIN PGP SIGNATURE-----") {
            // ASCII-armored signature
            let sig_str =
                String::from_utf8(sig_data).context("Failed to convert signature to string")?;
            let (sig, _headers) = DetachedSignature::from_string(&sig_str)
                .context("Failed to parse ASCII-armored signature")?;
            sig
        } else {
            // Binary signature
            DetachedSignature::from_bytes(Cursor::new(&sig_data[..]))
                .context("Failed to parse binary signature")?
        };

        info!("Verifying signature...");
        signature
            .verify(&public_key, &asset_data[..])
            .context("GPG signature verification failed")?;

        info!("GPG signature verification passed!");
        Ok(())
    }

    fn download_data(&self, url: &str) -> Result<Vec<u8>> {
        let response = self
            .client
            .get(url)
            .header("User-Agent", "picolayer")
            .send()
            .context("Failed to download")?;

        if !response.status().is_success() {
            anyhow::bail!("Download failed: {}", response.status());
        }

        Ok(response.bytes()?.to_vec())
    }

    fn load_public_key(&self, key_content: &str) -> Result<pgp::composed::SignedPublicKey> {
        use pgp::composed::{Deserializable, SignedPublicKey};

        let key_data = if key_content.starts_with("http://") || key_content.starts_with("https://")
        {
            info!("Downloading GPG public key from URL...");
            let response = self
                .client
                .get(key_content)
                .header("User-Agent", "picolayer")
                .send()
                .context("Failed to download GPG public key")?;

            if !response.status().is_success() {
                anyhow::bail!("Failed to download GPG public key: {}", response.status());
            }

            response
                .text()
                .context("Failed to read GPG public key response")?
        } else if std::path::Path::new(key_content).exists() {
            std::fs::read_to_string(key_content).context("Failed to read GPG key file")?
        } else {
            key_content.to_string()
        };

        let (public_key, _headers) =
            SignedPublicKey::from_string(&key_data).context("Failed to parse GPG public key")?;

        Ok(public_key)
    }
}

fn get_filename_variants(filename: &str) -> Vec<String> {
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

fn parse_checksum_line(content: &str, asset_name: &str) -> Result<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.contains(asset_name) {
            #[allow(clippy::collapsible_if)]
            if let Some(hash) = line.split_whitespace().next() {
                return Ok(hash.to_string());
            }
        }
    }

    anyhow::bail!("Checksum not found for asset: {}", asset_name)
}

fn compute_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_sha256() {
        let data = b"hello world";
        let hash = compute_sha256(data);
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_get_filename_variants() {
        let variants = get_filename_variants("file.tar.gz");
        assert!(variants.contains(&"file.tar.gz".to_string()));
        assert!(variants.contains(&"file".to_string()));
    }

    #[test]
    fn test_parse_checksum_line_success() {
        let content = "abc123def456  file.tar.gz\n";
        let result = parse_checksum_line(content, "file.tar.gz");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "abc123def456");
    }

    #[test]
    fn test_parse_checksum_line_not_found() {
        let content = "abc123def456  other-file.tar.gz\n";
        let result = parse_checksum_line(content, "nonexistent-file.tar.gz");
        assert!(result.is_err());
    }

    #[test]
    fn test_asset_selector_arch_patterns() {
        let selector = AssetSelector::new();
        let patterns = selector.get_arch_patterns("x86_64");
        assert!(patterns.contains(&"x86_64"));
        assert!(patterns.contains(&"amd64"));
    }

    #[test]
    fn test_asset_selector_os_patterns() {
        let selector = AssetSelector::new();
        let patterns = selector.get_os_patterns("linux");
        assert!(patterns.contains(&"linux"));
        assert!(patterns.contains(&"Linux"));
    }

    #[test]
    fn test_asset_selector_is_archive() {
        let selector = AssetSelector::new();
        assert!(selector.is_archive("file.tar.gz"));
        assert!(selector.is_archive("file.tgz"));
        assert!(selector.is_archive("file.tar.xz"));
        assert!(selector.is_archive("file.zip"));
        assert!(!selector.is_archive("file.txt"));
    }

    #[test]
    fn test_asset_installer_is_tar_xz_archive() {
        let client = Client::new();
        let installer = AssetInstaller::new(&client);
        
        // XZ magic bytes: 0xFD, '7', 'z', 'X', 'Z', 0x00
        let xz_data = vec![0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00];
        assert!(installer.is_tar_xz_archive(&xz_data));
        
        let not_xz_data = vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05];
        assert!(!installer.is_tar_xz_archive(&not_xz_data));
    }
}
