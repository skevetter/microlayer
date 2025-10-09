use anyhow::Result;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// Parse OCI reference string
pub fn parse_oci_ref(oci_input: &str) -> Result<ParsedOciRef> {
    let oci_input = oci_input.replace("http://", "").replace("https://", "");
    
    // Find version separator
    let index_of_last_colon = oci_input.rfind(':');
    let (resource, version) = if let Some(idx) = index_of_last_colon {
        if idx < oci_input.find('/').unwrap_or(usize::MAX) {
            (oci_input.clone(), "latest".to_string())
        } else {
            (oci_input[..idx].to_string(), oci_input[idx + 1..].to_string())
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
    let namespace = parts[1..parts.len()-1].join("/");
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

/// Install a devcontainer feature from an OCI reference
pub fn install(
    feature_ref: &str,
    _options: Option<HashMap<String, String>>,
    _remote_user: Option<&str>,
    _envs: Option<HashMap<String, String>>,
) -> Result<()> {
    info!("Installing devcontainer feature: {}", feature_ref);
    
    // Check for root privileges
    if !crate::utils::command::is_elevated() {
        anyhow::bail!("Devcontainer feature installation requires root privileges. Use sudo.");
    }
    
    let parsed_ref = parse_oci_ref(feature_ref)?;
    debug!("Parsed OCI ref: {:?}", parsed_ref);
    
    // For now, provide a basic implementation message
    // Full implementation requires OCI registry interaction
    warn!("Devcontainer feature installation is not yet fully implemented");
    warn!("This would download and install: {}", parsed_ref.resource);
    
    // TODO: Implement full OCI download and feature installation
    // 1. Download manifest from OCI registry
    // 2. Download feature layers
    // 3. Extract to temporary directory
    // 4. Execute install.sh with proper environment
    // 5. Handle remote user resolution
    // 6. Set up environment variables
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
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
    fn test_parse_oci_ref_without_version() {
        let result = parse_oci_ref("ghcr.io/devcontainers/features/node");
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.id, "node");
        assert_eq!(parsed.version, "latest");
    }

    #[test]
    fn test_parse_oci_ref_with_multiple_namespace_parts() {
        let result = parse_oci_ref("ghcr.io/devcontainers-contrib/features/go-task:1");
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.id, "go-task");
        assert_eq!(parsed.version, "1");
        assert_eq!(parsed.namespace, "devcontainers-contrib/features");
    }

    #[test]
    fn test_parse_oci_ref_invalid() {
        let result = parse_oci_ref("invalid");
        assert!(result.is_err());
    }
}
