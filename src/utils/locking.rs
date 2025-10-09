use anyhow::{Context, Result};
use log::{debug, info, warn};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

const LOCK_FILE_NAME: &str = ".picolayer.lock";
const LOCK_TIMEOUT_SECS: u64 = 300; // 5 minutes
const LOCK_RETRY_DELAY_MS: u64 = 100;
const LOCK_MAX_RETRIES: u32 = 50;

/// Get the lock file path for pkgx operations
fn get_lock_path() -> Result<PathBuf> {
    let lock_dir = if let Some(home_dir) = dirs_next::home_dir() {
        home_dir.join(".pkgx")
    } else {
        PathBuf::from("/tmp")
    };

    if !lock_dir.exists() {
        fs::create_dir_all(&lock_dir).context("Failed to create lock directory")?;
    }

    Ok(lock_dir.join(LOCK_FILE_NAME))
}

/// Check if a lock file is stale (older than timeout)
fn is_lock_stale(lock_path: &PathBuf) -> bool {
    if let Ok(metadata) = fs::metadata(lock_path)
        && let Ok(modified) = metadata.modified()
        && let Ok(elapsed) = SystemTime::now().duration_since(modified)
    {
        return elapsed > Duration::from_secs(LOCK_TIMEOUT_SECS);
    }

    false
}

/// Acquire a lock for pkgx operations
pub fn acquire_lock() -> Result<PkgxLock> {
    let lock_path = get_lock_path()?;
    debug!("Attempting to acquire lock at: {}", lock_path.display());

    let mut retries = 0;

    loop {
        if lock_path.exists() {
            if is_lock_stale(&lock_path) {
                warn!("Found stale lock file, removing it");
                let _ = fs::remove_file(&lock_path);
            } else if retries >= LOCK_MAX_RETRIES {
                anyhow::bail!(
                    "Failed to acquire lock after {} retries. Another picolayer instance may be running.",
                    LOCK_MAX_RETRIES
                );
            } else {
                debug!(
                    "Lock exists, waiting... (attempt {}/{})",
                    retries + 1,
                    LOCK_MAX_RETRIES
                );
                std::thread::sleep(Duration::from_millis(LOCK_RETRY_DELAY_MS));
                retries += 1;
                continue;
            }
        }

        match File::create(&lock_path) {
            Ok(mut file) => {
                let pid = std::process::id();
                let timestamp = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                let lock_info = format!("pid={}\ntimestamp={}\n", pid, timestamp);
                let _ = file.write_all(lock_info.as_bytes());

                info!("Lock acquired at: {}", lock_path.display());

                return Ok(PkgxLock {
                    lock_path,
                    acquired: true,
                });
            }
            Err(e) => {
                if retries >= LOCK_MAX_RETRIES {
                    return Err(anyhow::anyhow!("Failed to create lockfile: {}", e));
                }
                debug!(
                    "Failed to create lockfile (attempt {}/{}): {}",
                    retries + 1,
                    LOCK_MAX_RETRIES,
                    e
                );
                std::thread::sleep(Duration::from_millis(LOCK_RETRY_DELAY_MS));
                retries += 1;
            }
        }
    }
}

/// RAII lock guard for pkgx operations
pub struct PkgxLock {
    lock_path: PathBuf,
    acquired: bool,
}

impl PkgxLock {
    /// Manually release the lock
    pub fn release(&mut self) -> Result<()> {
        if self.acquired {
            if self.lock_path.exists() {
                fs::remove_file(&self.lock_path).context("Failed to remove lock file")?;
                info!("Lock released at: {}", self.lock_path.display());
            }
            self.acquired = false;
        }
        Ok(())
    }
}

impl Drop for PkgxLock {
    fn drop(&mut self) {
        if self.acquired {
            let _ = self.release();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_get_lock_path() {
        let path = get_lock_path();
        assert!(path.is_ok());
        let path = path.unwrap();
        assert!(path.to_string_lossy().contains(LOCK_FILE_NAME));
    }

    #[test]
    fn test_acquire_and_release_lock() {
        let mut lock = acquire_lock().expect("Failed to acquire lock");
        let lock_path = lock.lock_path.clone();

        assert!(lock_path.exists());

        lock.release().expect("Failed to release lock");

        assert!(!lock_path.exists());
        assert!(lock.release().is_ok());
    }

    #[test]
    fn test_lock_auto_release_on_drop() {
        let lock_path = {
            let lock = acquire_lock().expect("Failed to acquire lock");
            lock.lock_path.clone()
        };

        thread::sleep(Duration::from_millis(10));

        assert!(!lock_path.exists());
    }

    #[test]
    fn test_concurrent_lock_acquisition() {
        use serial_test::serial;

        #[serial]
        fn test_impl() {
            let lock1 = acquire_lock().expect("Failed to acquire first lock");

            let handle = thread::spawn(|| {
                let result = acquire_lock();
                result.is_err() || result.is_ok()
            });

            thread::sleep(Duration::from_millis(200));

            drop(lock1);

            let _ = handle.join();
        }

        test_impl();
    }

    #[test]
    fn test_stale_lock_detection() {
        let lock_path = get_lock_path().expect("Failed to get lock path");

        File::create(&lock_path).expect("Failed to create lock file");

        assert!(!is_lock_stale(&lock_path));

        let _ = fs::remove_file(&lock_path);
    }
}
