use anyhow::{Context, Result};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref LOG_FILE: Mutex<Option<std::fs::File>> = Mutex::new(None);
}

/// Initialize logging with optional file output
pub fn init_logging() -> Result<()> {
    // Initialize env_logger for stdout/stderr
    env_logger::init();
    
    // Check if file logging is enabled
    if let Ok(log_file_path) = std::env::var("PICOLAYER_LOG_FILE") {
        let path = PathBuf::from(&log_file_path);
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create log directory: {}", parent.display()))?;
        }
        
        // Open or create the log file
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .with_context(|| format!("Failed to open log file: {}", path.display()))?;
        
        let mut log_file = LOG_FILE.lock().unwrap();
        *log_file = Some(file);
        
        // Log initialization message
        log_to_file(&format!("=== picolayer started at {} ===\n", 
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")))?;
    }
    
    Ok(())
}

/// Log a message to the file (if configured)
pub fn log_to_file(message: &str) -> Result<()> {
    let mut log_file_guard = LOG_FILE.lock().unwrap();
    if let Some(ref mut file) = *log_file_guard {
        file.write_all(message.as_bytes())
            .context("Failed to write to log file")?;
        file.write_all(b"\n")
            .context("Failed to write newline to log file")?;
        file.flush()
            .context("Failed to flush log file")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_to_file_without_init() {
        // Should not panic even if not initialized
        let result = log_to_file("test message");
        assert!(result.is_ok());
    }
}
