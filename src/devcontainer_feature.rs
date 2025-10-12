use anyhow::{Context, Result};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

const ORDERED_BASE_USERS: &[&str] = &["vscode", "node", "codespace"];

/// OCI reference parser
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedOciRef {
    pub id: String,
    pub version: String,
    pub owner: String,
    pub namespace: String,
    pub registry: String,
    pub resource: String,
    pub path: String,
}

/// DevContainer Feature metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Feature {
    pub id: String,
    pub version: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub options: Option<HashMap<String, FeatureOption>>,
    pub container_env: Option<HashMap<String, String>>,
    pub entrypoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureOption {
    #[serde(rename = "type")]
    pub option_type: String,
    pub default: Option<serde_json::Value>,
    pub description: Option<String>,
}

/// OCI Manifest structure
#[derive(Debug, Deserialize)]
struct OciManifest {
    layers: Vec<OciLayer>,
}

#[derive(Debug, Deserialize)]
struct OciLayer {
    digest: String,
    #[allow(dead_code)]
    annotations: Option<HashMap<String, String>>,
}

/// Parse OCI reference string
pub fn parse_oci_ref(oci_input: &str) -> Result<ParsedOciRef> {
    let oci_input = oci_input.replace("http://", "").replace("https://", "");

    let index_of_last_colon = oci_input.rfind(':');
    let (resource, version) = if let Some(idx) = index_of_last_colon {
        if idx < oci_input.find('/').unwrap_or(usize::MAX) {
            (oci_input.clone(), "latest".to_string())
        } else {
            (
                oci_input[..idx].to_string(),
                oci_input[idx + 1..].to_string(),
            )
        }
    } else {
        (oci_input.clone(), "latest".to_string())
    };

    let parts: Vec<&str> = resource.split('/').collect();
    if parts.len() < 2 {
        anyhow::bail!("Invalid OCI reference format: {}", oci_input);
    }

    let id = parts.last().unwrap().to_string();
    let registry = parts[0].to_string();
    let owner = parts[1].to_string();
    let namespace = parts[1..parts.len() - 1].join("/");
    let path = format!("{}/{}", namespace, id);

    Ok(ParsedOciRef {
        id,
        version,
        owner,
        namespace,
        registry,
        resource,
        path,
    })
}

/// Get OCI manifest from registry
fn get_manifest(parsed_ref: &ParsedOciRef) -> Result<OciManifest> {
    let url = format!(
        "https://{}/v2/{}/manifests/{}",
        parsed_ref.registry, parsed_ref.path, parsed_ref.version
    );

    debug!("Fetching manifest from: {}", url);

    let client = reqwest::blocking::Client::new();
    let response = client
        .get(&url)
        .header("Accept", "application/vnd.oci.image.manifest.v1+json, application/vnd.docker.distribution.manifest.v2+json")
        .header("User-Agent", "picolayer")
        .send()
        .context("Failed to fetch OCI manifest")?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to fetch manifest: HTTP {}", response.status());
    }

    let manifest: OciManifest = response.json().context("Failed to parse manifest JSON")?;

    Ok(manifest)
}

/// Download and extract OCI layer
fn download_and_extract_layer(parsed_ref: &ParsedOciRef, output_dir: &Path) -> Result<()> {
    let manifest = get_manifest(parsed_ref)?;

    if manifest.layers.is_empty() {
        anyhow::bail!("Feature OCI manifest has no layers");
    }

    let layer = &manifest.layers[0];
    let blob_url = format!(
        "https://{}/v2/{}/blobs/{}",
        parsed_ref.registry, parsed_ref.path, layer.digest
    );

    debug!("Downloading layer from: {}", blob_url);

    let client = reqwest::blocking::Client::new();
    let response = client
        .get(&blob_url)
        .header("User-Agent", "picolayer")
        .send()
        .context("Failed to download layer blob")?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to download blob: HTTP {}", response.status());
    }

    let blob_data = response.bytes().context("Failed to read blob data")?;

    let decoder = flate2::read::GzDecoder::new(&blob_data[..]);
    let mut archive = tar::Archive::new(decoder);
    archive
        .unpack(output_dir)
        .context("Failed to extract layer archive")?;

    Ok(())
}

fn resolve_remote_user(remote_user: Option<&str>) -> Result<(String, String)> {
    if let Some(user) = remote_user
        && let Ok(output) = Command::new("id").arg("-u").arg(user).output()
        && output.status.success()
    {
        if let Ok(home) = std::env::var("HOME") {
            return Ok((user.to_string(), home));
        }
        if let Ok(output) = Command::new("sh")
            .arg("-c")
            .arg(format!("eval echo ~{}", user))
            .output()
            && output.status.success()
        {
            let home = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return Ok((user.to_string(), home));
        }

        warn!("User '{}' not found, attempting fallback", user);
    }

    for user in ORDERED_BASE_USERS {
        if let Ok(output) = Command::new("id").arg("-u").arg(user).output()
            && output.status.success()
            && let Ok(output) = Command::new("sh")
                .arg("-c")
                .arg(format!("eval echo ~{}", user))
                .output()
            && output.status.success()
        {
            let home = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return Ok((user.to_string(), home));
        }
    }

    // Fallback to user 1000
    if let Ok(output) = Command::new("id").arg("-un").arg("1000").output()
        && output.status.success()
    {
        let user = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if let Ok(output) = Command::new("sh")
            .arg("-c")
            .arg(format!("eval echo ~{}", user))
            .output()
            && output.status.success()
        {
            let home = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return Ok((user, home));
        }
    }

    Ok(("root".to_string(), "/root".to_string()))
}

/// Load feature metadata from directory
fn load_feature_metadata(feature_dir: &Path) -> Result<Feature> {
    let metadata_path = feature_dir.join("devcontainer-feature.json");

    if !metadata_path.exists() {
        anyhow::bail!("Feature metadata file not found: devcontainer-feature.json");
    }

    let metadata_content =
        fs::read_to_string(&metadata_path).context("Failed to read feature metadata")?;

    let feature: Feature =
        serde_json::from_str(&metadata_content).context("Failed to parse feature metadata")?;

    Ok(feature)
}

/// Resolve feature options with defaults
fn resolve_options(
    feature: &Feature,
    provided_options: Option<HashMap<String, String>>,
) -> HashMap<String, String> {
    let mut resolved = provided_options.unwrap_or_default();

    if let Some(option_defs) = &feature.options {
        for (name, option) in option_defs {
            if !resolved.contains_key(name)
                && let Some(default) = &option.default
            {
                let default_str = match default {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Bool(b) => if *b { "true" } else { "false" }.to_string(),
                    serde_json::Value::Number(n) => n.to_string(),
                    _ => String::new(),
                };
                resolved.insert(name.clone(), default_str);
            }
        }
    }

    resolved
}

/// Set environment variables in profile
fn set_container_env(feature: &Feature) -> Result<()> {
    const PROFILE_DIR: &str = "/etc/profile.d";

    if feature.container_env.is_none() {
        return Ok(());
    }

    let profile_dir = Path::new(PROFILE_DIR);
    fs::create_dir_all(profile_dir).context("Failed to create profile directory")?;

    let profile_file = profile_dir.join(format!("picolayer-{}.sh", feature.id));

    let mut content = String::new();
    if profile_file.exists() {
        content = fs::read_to_string(&profile_file)?;
    }

    if let Some(container_env) = &feature.container_env {
        for (key, value) in container_env {
            let statement = format!("export {}={}\n", key, value);
            if !content.contains(&statement) {
                content.push_str(&statement);
            }
        }
    }

    fs::write(&profile_file, content).context("Failed to write profile file")?;

    Ok(())
}

/// Install a devcontainer feature from an OCI reference
pub fn install(
    feature_ref: &str,
    options: Option<HashMap<String, String>>,
    remote_user: Option<&str>,
    envs: Option<HashMap<String, String>>,
) -> Result<()> {
    info!("Installing devcontainer feature: {}", feature_ref);

    let parsed_ref = parse_oci_ref(feature_ref)?;
    debug!("Parsed OCI ref: {:?}", parsed_ref);

    let temp_dir = tempfile::tempdir().context("Failed to create temporary directory")?;

    info!("Downloading and extracting feature...");
    download_and_extract_layer(&parsed_ref, temp_dir.path())?;

    let feature = load_feature_metadata(temp_dir.path())?;
    info!(
        "Feature: {} v{}",
        feature.id,
        feature.version.as_deref().unwrap_or("unknown")
    );

    let (remote_user_name, remote_user_home) = resolve_remote_user(remote_user)?;
    info!(
        "Installing for user: {} (home: {})",
        remote_user_name, remote_user_home
    );

    let resolved_options = resolve_options(&feature, options);
    debug!("Resolved options: {:?}", resolved_options);

    let mut env_vars = envs.unwrap_or_default();
    env_vars.insert("_REMOTE_USER".to_string(), remote_user_name.clone());
    env_vars.insert("_REMOTE_USER_HOME".to_string(), remote_user_home.clone());

    for (key, value) in resolved_options {
        env_vars.insert(key.to_uppercase(), value);
    }

    let install_script = temp_dir.path().join("install.sh");
    if !install_script.exists() {
        anyhow::bail!("Feature install.sh script not found");
    }

    info!("Executing feature installation script...");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&install_script)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&install_script, perms)?;
    }

    let env_string: Vec<String> = env_vars
        .iter()
        .map(|(k, v)| format!("{}=\"{}\"", k, v.replace("\"", "\\\"")))
        .collect();

    let env_prefix = env_string.join(" ");
    let command = format!(
        "cd {} && {} bash -i +H -x ./install.sh",
        temp_dir.path().display(),
        env_prefix
    );

    debug!("Executing: {}", command);

    let output = Command::new("sh")
        .arg("-c")
        .arg(&command)
        .output()
        .context("Failed to execute install script")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        warn!("Script output:\n{}", stdout);
        warn!("Script errors:\n{}", stderr);
        anyhow::bail!(
            "Feature installation script failed with exit code: {:?}",
            output.status.code()
        );
    }

    info!("Feature installation script completed successfully");

    set_container_env(&feature)?;

    if let Some(entrypoint) = &feature.entrypoint {
        info!("Executing feature entrypoint: {}", entrypoint);
        let output = Command::new("sh")
            .arg("-c")
            .arg(entrypoint)
            .output()
            .context("Failed to execute entrypoint")?;

        if !output.status.success() {
            warn!(
                "Entrypoint failed but continuing: {:?}",
                output.status.code()
            );
        }
    }

    info!("Devcontainer feature installation completed successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_parse_oci_ref_with_version() {
        let result = parse_oci_ref("ghcr.io/devcontainers/features/node:1.0.0");
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.id, "node");
        assert_eq!(parsed.version, "1.0.0");
        assert_eq!(parsed.registry, "ghcr.io");
        assert_eq!(parsed.owner, "devcontainers");
    }

    #[test]
    #[serial]
    fn test_parse_oci_ref_without_version() {
        let result = parse_oci_ref("ghcr.io/devcontainers/features/node");
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.id, "node");
        assert_eq!(parsed.version, "latest");
    }

    #[test]
    #[serial]
    fn test_parse_oci_ref_with_multiple_namespace_parts() {
        let result = parse_oci_ref("ghcr.io/devcontainers-contrib/features/go-task:1");
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.id, "go-task");
        assert_eq!(parsed.version, "1");
        assert_eq!(parsed.namespace, "devcontainers-contrib/features");
    }

    #[test]
    #[serial]
    fn test_parse_oci_ref_invalid() {
        let result = parse_oci_ref("invalid");
        assert!(result.is_err());
    }

    #[test]
    #[serial]
    fn test_resolve_options_with_defaults() {
        let mut options_def = HashMap::new();
        options_def.insert(
            "version".to_string(),
            FeatureOption {
                option_type: "string".to_string(),
                default: Some(serde_json::Value::String("18".to_string())),
                description: None,
            },
        );

        let feature = Feature {
            id: "test".to_string(),
            version: Some("1.0".to_string()),
            name: None,
            description: None,
            options: Some(options_def),
            container_env: None,
            entrypoint: None,
        };

        let resolved = resolve_options(&feature, None);
        assert_eq!(resolved.get("version"), Some(&"18".to_string()));
    }

    #[test]
    #[serial]
    fn test_resolve_options_override() {
        let mut options_def = HashMap::new();
        options_def.insert(
            "version".to_string(),
            FeatureOption {
                option_type: "string".to_string(),
                default: Some(serde_json::Value::String("18".to_string())),
                description: None,
            },
        );

        let feature = Feature {
            id: "test".to_string(),
            version: Some("1.0".to_string()),
            name: None,
            description: None,
            options: Some(options_def),
            container_env: None,
            entrypoint: None,
        };

        let mut provided = HashMap::new();
        provided.insert("version".to_string(), "20".to_string());

        let resolved = resolve_options(&feature, Some(provided));
        assert_eq!(resolved.get("version"), Some(&"20".to_string()));
    }
}
