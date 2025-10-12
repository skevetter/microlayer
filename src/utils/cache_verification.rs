use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Compute SHA256 hash of a directory's contents
pub fn compute_directory_hash(dir_path: &Path) -> Result<String> {
    let mut hasher = Sha256::new();
    
    // Sort entries for consistent hashing
    let mut entries: Vec<_> = WalkDir::new(dir_path)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|e| e.ok())
        .collect();
    
    entries.sort_by(|a, b| a.path().cmp(b.path()));
    
    for entry in entries {
        let path = entry.path();
        
        // Add relative path to hash
        if let Ok(rel_path) = path.strip_prefix(dir_path) {
            hasher.update(rel_path.to_string_lossy().as_bytes());
        }
        
        // Add file contents to hash
        if path.is_file() {
            let contents = fs::read(path)
                .with_context(|| format!("Failed to read file: {}", path.display()))?;
            hasher.update(&contents);
        }
    }
    
    Ok(format!("{:x}", hasher.finalize()))
}

/// Verify that a directory's hash matches the expected hash
pub fn verify_directory_hash(dir_path: &Path, expected_hash: &str) -> Result<bool> {
    let actual_hash = compute_directory_hash(dir_path)?;
    Ok(actual_hash == expected_hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_compute_directory_hash() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, b"test content").unwrap();
        
        let hash = compute_directory_hash(temp_dir.path()).unwrap();
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // SHA256 produces 64 hex characters
    }
    
    #[test]
    fn test_verify_directory_hash() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, b"test content").unwrap();
        
        let hash = compute_directory_hash(temp_dir.path()).unwrap();
        let is_valid = verify_directory_hash(temp_dir.path(), &hash).unwrap();
        assert!(is_valid);
        
        // Modify file and verify hash changes
        fs::write(&file_path, b"modified content").unwrap();
        let is_valid = verify_directory_hash(temp_dir.path(), &hash).unwrap();
        assert!(!is_valid);
    }
    
    #[test]
    fn test_consistent_hashing() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("a.txt"), b"content a").unwrap();
        fs::write(temp_dir.path().join("b.txt"), b"content b").unwrap();
        
        let hash1 = compute_directory_hash(temp_dir.path()).unwrap();
        let hash2 = compute_directory_hash(temp_dir.path()).unwrap();
        
        assert_eq!(hash1, hash2);
    }
}
