use std::path::PathBuf;

/// Centralized configuration for picolayer program settings
pub struct Config {
    /// Base directory for temporary files
    #[allow(dead_code)]
    pub temp_dir_prefix: &'static str,
    
    /// Lock file directory
    pub lock_dir: &'static str,
    
    /// Enable hash verification for cache backups
    pub verify_cache_backups: bool,
    
    /// Log file path (if enabled)
    pub log_file_path: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            temp_dir_prefix: "/tmp/picolayer",
            lock_dir: "/tmp/picolayer",
            verify_cache_backups: true,
            log_file_path: None,
        }
    }
}

impl Config {
    /// Get the global configuration instance
    pub fn global() -> Self {
        let mut config = Self::default();
        
        // Enable file logging if PICOLAYER_LOG_FILE is set
        if let Ok(log_file) = std::env::var("PICOLAYER_LOG_FILE") {
            config.log_file_path = Some(PathBuf::from(log_file));
        }
        
        // Optionally disable cache verification
        if let Ok(val) = std::env::var("PICOLAYER_VERIFY_CACHE") {
            config.verify_cache_backups = val != "0" && val.to_lowercase() != "false";
        }
        
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.temp_dir_prefix, "/tmp/picolayer");
        assert_eq!(config.lock_dir, "/tmp/picolayer");
        assert!(config.verify_cache_backups);
        assert!(config.log_file_path.is_none());
    }

    #[test]
    fn test_global_config() {
        let config = Config::global();
        assert_eq!(config.temp_dir_prefix, "/tmp/picolayer");
    }
}
