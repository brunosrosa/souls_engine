//! Incremental SHA-256 Checksum Validator.
//!
//! Provides memory-efficient block hashing for multi-gigabyte GGUF weights,
//! verifying file integrity against Hugging Face Git LFS checksums.

use crate::error::InventoryError;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// Memory-efficient buffer size for streaming disk I/O (64 KB).
const HASH_BUFFER_SIZE: usize = 64 * 1024;

/// Incremental SHA-256 hasher for streaming downloads.
#[derive(Default)]
pub struct IncrementalSha256 {
    hasher: Sha256,
}

impl IncrementalSha256 {
    /// Creates a new streaming hasher.
    pub fn new() -> Self {
        Self {
            hasher: Sha256::new(),
        }
    }

    /// Feeds a chunk of downloaded bytes into the hash computation.
    pub fn update(&mut self, chunk: &[u8]) {
        self.hasher.update(chunk);
    }

    /// Finalizes the hash and returns the lowercase hexadecimal string.
    pub fn finalize_hex(self) -> String {
        let result = self.hasher.finalize();
        format!("{:x}", result)
    }
}

/// Computes the SHA-256 hexadecimal hash of a file on disk without loading it into RAM.
pub fn compute_file_sha256(path: &Path) -> Result<String, InventoryError> {
    let file = File::open(path).map_err(|e| {
        InventoryError::CatalogIoError(path.display().to_string(), e.to_string())
    })?;

    let mut reader = BufReader::with_capacity(HASH_BUFFER_SIZE, file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; HASH_BUFFER_SIZE];

    loop {
        let bytes_read = reader.read(&mut buffer).map_err(|e| {
            InventoryError::CatalogIoError(path.display().to_string(), e.to_string())
        })?;

        if bytes_read == 0 {
            break;
        }

        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

/// Verifies whether the SHA-256 hash of a file matches the expected value.
pub fn verify_file_sha256(path: &Path, expected_hex: &str) -> Result<bool, InventoryError> {
    let computed = compute_file_sha256(path)?;
    let matches = computed.eq_ignore_ascii_case(expected_hex.trim());

    if !matches {
        return Err(InventoryError::ChecksumMismatch(
            path.display().to_string(),
            expected_hex.to_string(),
            computed,
        ));
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_incremental_sha256() {
        let mut hasher = IncrementalSha256::new();
        hasher.update(b"Souls");
        hasher.update(b"Engine");
        let hash = hasher.finalize_hex();

        // SHA-256 of "SoulsEngine"
        let mut direct = Sha256::new();
        direct.update(b"SoulsEngine");
        let expected = format!("{:x}", direct.finalize());

        assert_eq!(hash, expected);
    }

    #[test]
    fn test_file_sha256_verification() {
        let mut temp = NamedTempFile::new().unwrap();
        temp.write_all(b"GGUF_TEST_TENSOR_BYTES").unwrap();
        temp.flush().unwrap();

        let path = temp.path();
        let computed = compute_file_sha256(path).unwrap();

        assert!(verify_file_sha256(path, &computed).unwrap());
        assert!(verify_file_sha256(path, "invalid_hash").is_err());
    }
}
