use anyhow::Result;
use log::debug;
use std::env;

/// Check if analytics is enabled via environment variable
pub fn is_analytics_enabled() -> bool {
    if let Ok(val) = env::var("PICOLAYER_NO_ANALYTICS")
        && (val == "1" || val.to_lowercase() == "true")
    {
        return false;
    }
    true
}

/// Get the PostHog API key from environment variable
fn get_api_key() -> Option<String> {
    env::var("PH_PICOLAYER_API_KEY").ok()
}

/// Track an event with analytics using direct HTTP call
pub fn track_event(event_name: &str, properties: Option<serde_json::Value>) -> Result<()> {
    if !is_analytics_enabled() {
        return Ok(());
    }

    let api_key = match get_api_key() {
        Some(key) => key,
        None => {
            debug!("PostHog API key not found (PH_PICOLAYER_API_KEY)");
            return Ok(());
        }
    };

    let distinct_id = get_distinct_id();

    let event_name_owned = event_name.to_string();
    let mut event_payload = serde_json::json!({
        "api_key": api_key,
        "event": event_name_owned,
        "distinct_id": distinct_id,
    });

    if let Some(props) = properties {
        event_payload["properties"] = props;
    }

    std::thread::spawn(move || {
        let client = reqwest::blocking::Client::new();
        match client
            .post("https://app.posthog.com/capture/")
            .json(&event_payload)
            .send()
        {
            Ok(_) => {
                debug!("Tracked event: {}", event_name_owned);
            }
            Err(e) => {
                debug!("Failed to track event {}: {}", event_name_owned, e);
            }
        }
    });

    Ok(())
}

/// Generate a distinct ID for analytics
/// Uses machine ID or hostname as fallback
fn get_distinct_id() -> String {
    if let Ok(machine_id) = get_machine_id() {
        return machine_id;
    }

    if let Ok(hostname) = hostname::get()
        && let Ok(hostname_str) = hostname.into_string()
    {
        return format!("hostname:{}", hostname_str);
    }

    format!("anonymous:{}", uuid::Uuid::new_v4())
}

/// Get machine ID (platform-specific)
#[cfg(target_os = "linux")]
fn get_machine_id() -> Result<String> {
    use std::fs;

    if let Ok(id) = fs::read_to_string("/etc/machine-id") {
        return Ok(id.trim().to_string());
    }

    if let Ok(id) = fs::read_to_string("/var/lib/dbus/machine-id") {
        return Ok(id.trim().to_string());
    }

    anyhow::bail!("Could not read machine ID")
}

#[cfg(target_os = "macos")]
fn get_machine_id() -> Result<String> {
    use std::process::Command;

    let output = Command::new("ioreg")
        .args(&["-rd1", "-c", "IOPlatformExpertDevice"])
        .output()?;

    let output_str = String::from_utf8_lossy(&output.stdout);

    for line in output_str.lines() {
        if line.contains("IOPlatformUUID") {
            if let Some(uuid) = line.split('"').nth(3) {
                return Ok(uuid.to_string());
            }
        }
    }

    anyhow::bail!("Could not extract IOPlatformUUID")
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn get_machine_id() -> Result<String> {
    anyhow::bail!("Machine ID not supported on this platform")
}

/// Track a command execution
pub fn track_command(command: &str, properties: Option<serde_json::Value>) -> Result<()> {
    let event_name = format!("command_{}", command);

    let mut props = properties.unwrap_or_else(|| serde_json::json!({}));

    if let Some(obj) = props.as_object_mut() {
        obj.insert("command".to_string(), serde_json::json!(command));
        obj.insert(
            "version".to_string(),
            serde_json::json!(env!("CARGO_PKG_VERSION")),
        );
        obj.insert("os".to_string(), serde_json::json!(std::env::consts::OS));
        obj.insert(
            "arch".to_string(),
            serde_json::json!(std::env::consts::ARCH),
        );
    }

    track_event(&event_name, Some(props))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_analytics_enabled_default() {
        let saved = env::var("PICOLAYER_NO_ANALYTICS").ok();

        unsafe {
            env::remove_var("PICOLAYER_NO_ANALYTICS");
        }

        assert!(is_analytics_enabled());

        if let Some(val) = saved {
            unsafe {
                env::set_var("PICOLAYER_NO_ANALYTICS", val);
            }
        }
    }

    #[test]
    fn test_is_analytics_enabled_disabled() {
        let saved = env::var("PICOLAYER_NO_ANALYTICS").ok();

        unsafe {
            env::set_var("PICOLAYER_NO_ANALYTICS", "1");
        }
        assert!(!is_analytics_enabled());

        unsafe {
            env::set_var("PICOLAYER_NO_ANALYTICS", "true");
        }
        assert!(!is_analytics_enabled());

        if let Some(val) = saved {
            unsafe {
                env::set_var("PICOLAYER_NO_ANALYTICS", val);
            }
        } else {
            unsafe {
                env::remove_var("PICOLAYER_NO_ANALYTICS");
            }
        }
    }

    #[test]
    fn test_get_distinct_id() {
        let id = get_distinct_id();
        assert!(!id.is_empty());
    }

    #[test]
    fn test_track_command_with_analytics_disabled() {
        let saved = env::var("PICOLAYER_NO_ANALYTICS").ok();
        unsafe {
            env::set_var("PICOLAYER_NO_ANALYTICS", "1");
        }

        let result = track_command("test", None);
        assert!(result.is_ok());

        if let Some(val) = saved {
            unsafe {
                env::set_var("PICOLAYER_NO_ANALYTICS", val);
            }
        } else {
            unsafe {
                env::remove_var("PICOLAYER_NO_ANALYTICS");
            }
        }
    }

    #[test]
    fn test_track_event_with_analytics_disabled() {
        let saved = env::var("PICOLAYER_NO_ANALYTICS").ok();
        unsafe {
            env::set_var("PICOLAYER_NO_ANALYTICS", "1");
        }

        let result = track_event("test_event", None);
        assert!(result.is_ok());

        if let Some(val) = saved {
            unsafe {
                env::set_var("PICOLAYER_NO_ANALYTICS", val);
            }
        } else {
            unsafe {
                env::remove_var("PICOLAYER_NO_ANALYTICS");
            }
        }
    }
}
