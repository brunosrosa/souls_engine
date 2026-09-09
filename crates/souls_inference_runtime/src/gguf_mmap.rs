//! Zero-copy O(1) GGUF file header metadata parser via virtual memory mapping (memmap2).
//!
//! Complies with ADR-005 and souls_inference_runtime specifications:
//! - Immediate O(1) inspection of architecture family, context window, tensor count, and blocks.
//! - Virtual address mapping without reading entire model weights into RAM.
//! - Strict adherence to unsafe justification policy: `// SAFETY: <racional>`.

use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use dashmap::DashMap;
use memmap2::Mmap;
use serde::{Deserialize, Serialize};

use crate::error::InferenceError;

/// Canonical upper limit of concurrently mapped GGUF file descriptors / handles.
pub const MAX_MAPPED_HANDLES: usize = 16;

/// GGUF file magic identifier (`GGUF` in ASCII).
pub const GGUF_MAGIC: [u8; 4] = *b"GGUF";

/// Structured metadata information extracted in O(1) from GGUF header.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GgufMetadataInfo {
    /// Model architecture family (e.g. "qwen2", "llama", "gemma2", "phi3").
    pub architecture: String,
    /// Context window size in tokens.
    pub context_length: u64,
    /// Embedding dimension.
    pub embedding_length: u64,
    /// Number of transformer blocks/layers.
    pub block_count: u64,
    /// Total tensor count in the file.
    pub tensor_count: u64,
    /// GGUF file format version (e.g. 2 or 3).
    pub version: u32,
    /// Total file size in bytes.
    pub file_size_bytes: u64,
    /// Total number of key-value metadata pairs stored in header.
    pub kv_pairs_count: u64,
}

/// Encapsulates a mapped GGUF file and its parsed metadata with explicit handle lifetime management.
///
/// On Windows NT / ReFS, holding an open memory mapping locks the underlying file descriptor,
/// preventing file operations (rename, overwrite, delete) with OS Error 5 (Access Denied).
/// `GgufMappedReader` allows explicit `unload(&mut self)` to release the OS handle immediately,
/// and guarantees handle release on `Drop`.
pub struct GgufMappedReader {
    mmap: Option<Mmap>,
    pub metadata: GgufMetadataInfo,
    path: std::path::PathBuf,
}

impl GgufMappedReader {
    /// Opens and memory-maps a GGUF file in O(1) time complexity.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, InferenceError> {
        let p = path.as_ref().to_path_buf();
        if !p.exists() {
            return Err(InferenceError::ModelNotFound(p.display().to_string()));
        }

        let file = File::open(&p)?;
        let file_size_bytes = file.metadata()?.len();

        if file_size_bytes < 24 {
            return Err(InferenceError::GgufParseError(
                "File is too small to contain a valid GGUF header".to_string(),
            ));
        }

        // SAFETY: We create a read-only memory map of the local GGUF file.
        // The mapped memory is accessed strictly as immutable slices and will not be modified.
        let mmap = unsafe { Mmap::map(&file)? };
        let metadata = parse_gguf_slice(&mmap, file_size_bytes)?;

        Ok(Self {
            mmap: Some(mmap),
            metadata,
            path: p,
        })
    }

    /// Explicitly unloads and drops the virtual memory map, immediately releasing the ReFS file handle.
    pub fn unload(&mut self) {
        if let Some(mmap) = self.mmap.take() {
            drop(mmap);
            tracing::info!("Explicitly unloaded GGUF memory map for: {:?}", self.path);
        }
    }

    /// Returns true if the memory map is currently active in virtual memory.
    pub fn is_loaded(&self) -> bool {
        self.mmap.is_some()
    }

    /// Returns a reference to the parsed metadata.
    pub fn metadata(&self) -> &GgufMetadataInfo {
        &self.metadata
    }

    /// Provides access to the mapped byte slice if loaded.
    pub fn as_slice(&self) -> Option<&[u8]> {
        self.mmap.as_deref()
    }

    /// Returns the path of the underlying GGUF model file.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for GgufMappedReader {
    fn drop(&mut self) {
        self.unload();
    }
}

impl std::fmt::Debug for GgufMappedReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GgufMappedReader")
            .field("path", &self.path)
            .field("is_loaded", &self.is_loaded())
            .field("metadata", &self.metadata)
            .finish()
    }
}

/// Internal entry in the bounded GGUF handle pool tracking recency.
#[derive(Debug)]
struct PoolEntry {
    reader: Arc<RwLock<GgufMappedReader>>,
    last_accessed: AtomicU64,
}

/// Thread-safe bounded pool of memory-mapped GGUF readers with LRU eviction.
///
/// Prevents Windows NT file descriptor / handle exhaustion (`EMFILE`) by enforcing a strict
/// upper limit (`MAX_MAPPED_HANDLES`) on open `GgufMappedReader` instances.
/// Hot-path lookups for already mapped files are lock-free and execute in O(1).
pub struct BoundedGgufPool {
    capacity: usize,
    entries: DashMap<PathBuf, Arc<PoolEntry>>,
    access_sequence: AtomicU64,
    eviction_lock: Mutex<()>,
}

impl BoundedGgufPool {
    /// Creates a new pool with the default capacity of `MAX_MAPPED_HANDLES` (16).
    pub fn new() -> Self {
        Self::with_capacity(MAX_MAPPED_HANDLES)
    }

    /// Creates a new pool with the specified maximum capacity of mapped handles.
    pub fn with_capacity(capacity: usize) -> Self {
        let cap = if capacity == 0 { 1 } else { capacity };
        Self {
            capacity: cap,
            entries: DashMap::new(),
            access_sequence: AtomicU64::new(1),
            eviction_lock: Mutex::new(()),
        }
    }

    /// Returns a global singleton instance of the bounded pool.
    pub fn global() -> &'static Self {
        static GLOBAL: OnceLock<BoundedGgufPool> = OnceLock::new();
        GLOBAL.get_or_init(BoundedGgufPool::new)
    }

    /// Retrieves or opens a memory-mapped GGUF reader for the given path.
    ///
    /// - If already mapped, updates the LRU access sequence and returns the existing reader in O(1).
    /// - If not mapped and the pool is at capacity, evicts the least recently used reader,
    ///   calling `.unload()` to release the physical NT kernel handle, before mapping the new file.
    pub fn get_or_open(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<Arc<RwLock<GgufMappedReader>>, InferenceError> {
        let key = path.as_ref().to_path_buf();

        // 1. Hot-path: Lock-free O(1) read lookup via sharded DashMap
        if let Some(entry) = self.entries.get(&key) {
            let seq = self.access_sequence.fetch_add(1, Ordering::Relaxed);
            entry.last_accessed.store(seq, Ordering::Relaxed);
            return Ok(Arc::clone(&entry.reader));
        }

        // Validate file exists before acquiring eviction lock to avoid unnecessary evictions
        if !key.exists() {
            return Err(InferenceError::ModelNotFound(key.display().to_string()));
        }

        // 2. Cold-path: Acquire eviction lock to serialize eviction and handle insertion
        let _guard = self.eviction_lock.lock().map_err(|_| {
            InferenceError::GgufParseError("Eviction lock poisoned".to_string())
        })?;

        // Double check if another thread inserted while we were waiting for the eviction lock
        if let Some(entry) = self.entries.get(&key) {
            let seq = self.access_sequence.fetch_add(1, Ordering::Relaxed);
            entry.last_accessed.store(seq, Ordering::Relaxed);
            return Ok(Arc::clone(&entry.reader));
        }

        // 3. Eviction: If at capacity, evict the least recently used reader(s)
        while self.entries.len() >= self.capacity && !self.entries.is_empty() {
            let mut lru_key: Option<PathBuf> = None;
            let mut oldest_seq = u64::MAX;

            for item in self.entries.iter() {
                let seq = item.value().last_accessed.load(Ordering::Relaxed);
                if seq < oldest_seq {
                    oldest_seq = seq;
                    lru_key = Some(item.key().clone());
                }
            }

            if let Some(victim_key) = lru_key {
                if let Some((_, victim_entry)) = self.entries.remove(&victim_key) {
                    // Explicitly unload virtual memory map to release NT kernel handle
                    match victim_entry.reader.write() {
                        Ok(mut writer) => writer.unload(),
                        Err(poisoned) => poisoned.into_inner().unload(),
                    }
                    tracing::info!(
                        victim = ?victim_key,
                        remaining = self.entries.len(),
                        capacity = self.capacity,
                        "BoundedGgufPool evicted and unloaded LRU handle"
                    );
                }
            } else {
                break;
            }
        }

        // 4. Map the new GGUF file
        let reader = GgufMappedReader::open(&key)?;
        let seq = self.access_sequence.fetch_add(1, Ordering::Relaxed);
        let entry = Arc::new(PoolEntry {
            reader: Arc::new(RwLock::new(reader)),
            last_accessed: AtomicU64::new(seq),
        });

        self.entries.insert(key, Arc::clone(&entry));
        Ok(Arc::clone(&entry.reader))
    }

    /// Inspects metadata of a GGUF file through the pool, caching the mapped handle.
    pub fn inspect_metadata(&self, path: impl AsRef<Path>) -> Result<GgufMetadataInfo, InferenceError> {
        let reader = self.get_or_open(path)?;
        let r = reader.read().map_err(|_| {
            InferenceError::GgufParseError("Reader lock poisoned".to_string())
        })?;
        Ok(r.metadata().clone())
    }

    /// Looks up a mapped reader in the pool if already loaded, updating its LRU timestamp.
    pub fn get(&self, path: impl AsRef<Path>) -> Option<Arc<RwLock<GgufMappedReader>>> {
        let key = path.as_ref().to_path_buf();
        if let Some(entry) = self.entries.get(&key) {
            let seq = self.access_sequence.fetch_add(1, Ordering::Relaxed);
            entry.last_accessed.store(seq, Ordering::Relaxed);
            Some(Arc::clone(&entry.reader))
        } else {
            None
        }
    }

    /// Checks if a GGUF model path is currently mapped in the pool without updating recency.
    pub fn contains(&self, path: impl AsRef<Path>) -> bool {
        self.entries.contains_key(path.as_ref())
    }

    /// Returns the number of currently mapped GGUF readers in the pool.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the pool contains no mapped readers.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the maximum capacity of mapped handles.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Counts the number of active entries whose memory mapping is currently valid.
    pub fn active_handles(&self) -> usize {
        self.entries
            .iter()
            .filter(|item| {
                if let Ok(guard) = item.value().reader.read() {
                    guard.is_loaded()
                } else {
                    false
                }
            })
            .count()
    }

    /// Explicitly unloads and removes a specific path from the pool.
    pub fn unload(&self, path: impl AsRef<Path>) -> bool {
        let key = path.as_ref().to_path_buf();
        if let Some((_, entry)) = self.entries.remove(&key) {
            match entry.reader.write() {
                Ok(mut r) => r.unload(),
                Err(p) => p.into_inner().unload(),
            }
            true
        } else {
            false
        }
    }

    /// Unloads and evicts all readers from the pool.
    pub fn clear(&self) {
        let _guard = self.eviction_lock.lock();
        for entry in self.entries.iter() {
            match entry.value().reader.write() {
                Ok(mut r) => r.unload(),
                Err(p) => p.into_inner().unload(),
            }
        }
        self.entries.clear();
    }
}

impl Default for BoundedGgufPool {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for BoundedGgufPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundedGgufPool")
            .field("capacity", &self.capacity)
            .field("len", &self.entries.len())
            .field("active_handles", &self.active_handles())
            .finish()
    }
}

/// Inspects GGUF metadata in O(1) time complexity via zero-copy read-only mmap.
pub fn inspect_gguf_metadata_o1(path: &Path) -> Result<GgufMetadataInfo, InferenceError> {
    let mut reader = GgufMappedReader::open(path)?;
    let metadata = reader.metadata.clone();
    reader.unload(); // Explicit early unload per Seguro A (ReFS safety)
    Ok(metadata)
}

/// Parses the GGUF header from an in-memory byte slice.
pub fn parse_gguf_slice(slice: &[u8], file_size_bytes: u64) -> Result<GgufMetadataInfo, InferenceError> {
    if slice.len() < 24 {
        return Err(InferenceError::GgufParseError(
            "Slice is smaller than minimum GGUF header size (24 bytes)".to_string(),
        ));
    }

    // 1. Magic bytes
    if slice[0..4] != GGUF_MAGIC {
        return Err(InferenceError::GgufParseError(format!(
            "Invalid GGUF magic bytes: expected {:?}, got {:?}",
            GGUF_MAGIC,
            &slice[0..4]
        )));
    }

    // 2. Version (u32, little-endian)
    let version = u32::from_le_bytes(
        slice[4..8]
            .try_into()
            .map_err(|_| InferenceError::GgufParseError("Failed reading version".into()))?,
    );

    if !(1..=4).contains(&version) {
        return Err(InferenceError::GgufParseError(format!(
            "Unsupported GGUF version: {}",
            version
        )));
    }

    // 3. Tensor count (u64, little-endian)
    let tensor_count = u64::from_le_bytes(
        slice[8..16]
            .try_into()
            .map_err(|_| InferenceError::GgufParseError("Failed reading tensor count".into()))?,
    );

    // 4. Metadata KV count (u64, little-endian)
    let kv_pairs_count = u64::from_le_bytes(
        slice[16..24]
            .try_into()
            .map_err(|_| InferenceError::GgufParseError("Failed reading KV count".into()))?,
    );

    let mut cursor = 24usize;
    let mut architecture = String::from("unknown");
    let mut context_length = 4096u64;
    let mut embedding_length = 2048u64;
    let mut block_count = 28u64;

    // 5. Parse KV entries up to the header boundary
    for _ in 0..kv_pairs_count {
        if cursor + 8 > slice.len() {
            break;
        }

        // Read key length (u64)
        let key_len = u64::from_le_bytes(
            slice[cursor..cursor + 8]
                .try_into()
                .map_err(|_| InferenceError::GgufParseError("Malformed key length".into()))?,
        ) as usize;
        cursor += 8;

        if cursor + key_len + 4 > slice.len() {
            break;
        }

        let key_str = std::str::from_utf8(&slice[cursor..cursor + key_len])
            .map_err(|e| InferenceError::GgufParseError(format!("Invalid UTF-8 in key: {}", e)))?;
        cursor += key_len;

        // Read value type (u32)
        let val_type = u32::from_le_bytes(
            slice[cursor..cursor + 4]
                .try_into()
                .map_err(|_| InferenceError::GgufParseError("Malformed value type".into()))?,
        );
        cursor += 4;

        // Extract key attributes
        if key_str == "general.architecture" && val_type == 8 {
            // String type
            if cursor + 8 <= slice.len() {
                let s_len = u64::from_le_bytes(slice[cursor..cursor + 8].try_into().unwrap_or([0; 8])) as usize;
                cursor += 8;
                if cursor + s_len <= slice.len() {
                    if let Ok(s) = std::str::from_utf8(&slice[cursor..cursor + s_len]) {
                        architecture = s.to_string();
                    }
                    cursor += s_len;
                    continue;
                }
            }
        } else if key_str.ends_with(".context_length") {
            if val_type == 4 && cursor + 4 <= slice.len() {
                context_length = u32::from_le_bytes(slice[cursor..cursor + 4].try_into().unwrap_or([0; 4])) as u64;
                cursor += 4;
                continue;
            } else if (val_type == 10 || val_type == 11) && cursor + 8 <= slice.len() {
                context_length = u64::from_le_bytes(slice[cursor..cursor + 8].try_into().unwrap_or([0; 8]));
                cursor += 8;
                continue;
            }
        } else if key_str.ends_with(".embedding_length") {
            if val_type == 4 && cursor + 4 <= slice.len() {
                embedding_length = u32::from_le_bytes(slice[cursor..cursor + 4].try_into().unwrap_or([0; 4])) as u64;
                cursor += 4;
                continue;
            } else if (val_type == 10 || val_type == 11) && cursor + 8 <= slice.len() {
                embedding_length = u64::from_le_bytes(slice[cursor..cursor + 8].try_into().unwrap_or([0; 8]));
                cursor += 8;
                continue;
            }
        } else if key_str.ends_with(".block_count") {
            if val_type == 4 && cursor + 4 <= slice.len() {
                block_count = u32::from_le_bytes(slice[cursor..cursor + 4].try_into().unwrap_or([0; 4])) as u64;
                cursor += 4;
                continue;
            } else if (val_type == 10 || val_type == 11) && cursor + 8 <= slice.len() {
                block_count = u64::from_le_bytes(slice[cursor..cursor + 8].try_into().unwrap_or([0; 8]));
                cursor += 8;
                continue;
            }
        }

        // Skip unknown value types cleanly
        cursor = skip_gguf_value(slice, cursor, val_type)?;
    }

    Ok(GgufMetadataInfo {
        architecture,
        context_length,
        embedding_length,
        block_count,
        tensor_count,
        version,
        file_size_bytes,
        kv_pairs_count,
    })
}

/// Helper to advance cursor past a GGUF value type.
fn skip_gguf_value(slice: &[u8], mut cursor: usize, val_type: u32) -> Result<usize, InferenceError> {
    match val_type {
        0 | 1 | 7 => cursor += 1, // uint8, int8, bool
        2 | 3 => cursor += 2,     // uint16, int16
        4..=6 => cursor += 4,     // uint32, int32, float32
        10..=12 => cursor += 8,   // uint64, int64, float64
        8 => {
            // string: len (u64) + bytes
            if cursor + 8 > slice.len() {
                return Err(InferenceError::GgufParseError("Unexpected EOF in string value".into()));
            }
            let s_len = u64::from_le_bytes(slice[cursor..cursor + 8].try_into().unwrap_or([0; 8])) as usize;
            cursor += 8 + s_len;
        }
        9 => {
            // array: item_type (u32) + len (u64) + items
            if cursor + 12 > slice.len() {
                return Err(InferenceError::GgufParseError("Unexpected EOF in array header".into()));
            }
            let item_type = u32::from_le_bytes(slice[cursor..cursor + 4].try_into().unwrap_or([0; 4]));
            let arr_len = u64::from_le_bytes(slice[cursor + 4..cursor + 12].try_into().unwrap_or([0; 8])) as usize;
            cursor += 12;

            for _ in 0..arr_len {
                cursor = skip_gguf_value(slice, cursor, item_type)?;
            }
        }
        _ => {
            return Err(InferenceError::GgufParseError(format!(
                "Unknown GGUF value type: {}",
                val_type
            )));
        }
    }

    if cursor > slice.len() {
        Err(InferenceError::GgufParseError("GGUF value exceeded slice bounds".into()))
    } else {
        Ok(cursor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_gguf_mmap_o1_parsing() {
        let mut file = NamedTempFile::new().unwrap();

        // Build a synthetic valid GGUF v3 header
        let mut buf = Vec::new();
        // Magic
        buf.extend_from_slice(&GGUF_MAGIC);
        // Version 3
        buf.extend_from_slice(&3u32.to_le_bytes());
        // Tensor count: 120
        buf.extend_from_slice(&120u64.to_le_bytes());
        // KV count: 2
        buf.extend_from_slice(&2u64.to_le_bytes());

        // KV 1: general.architecture (string = "qwen2")
        let key1 = b"general.architecture";
        buf.extend_from_slice(&(key1.len() as u64).to_le_bytes());
        buf.extend_from_slice(key1);
        buf.extend_from_slice(&8u32.to_le_bytes()); // type 8 = string
        let val1 = b"qwen2";
        buf.extend_from_slice(&(val1.len() as u64).to_le_bytes());
        buf.extend_from_slice(val1);

        // KV 2: qwen2.context_length (u32 = 8192)
        let key2 = b"qwen2.context_length";
        buf.extend_from_slice(&(key2.len() as u64).to_le_bytes());
        buf.extend_from_slice(key2);
        buf.extend_from_slice(&4u32.to_le_bytes()); // type 4 = uint32
        buf.extend_from_slice(&8192u32.to_le_bytes());

        // Pad with dummy bytes
        buf.extend_from_slice(&[0u8; 128]);

        file.write_all(&buf).unwrap();

        let info = inspect_gguf_metadata_o1(file.path()).expect("Failed to parse GGUF metadata");

        assert_eq!(info.version, 3);
        assert_eq!(info.architecture, "qwen2");
        assert_eq!(info.context_length, 8192);
        assert_eq!(info.tensor_count, 120);
        assert_eq!(info.kv_pairs_count, 2);
    }

    #[test]
    fn test_gguf_mapped_reader_explicit_unload() {
        let mut file = NamedTempFile::new().unwrap();

        let mut buf = Vec::new();
        buf.extend_from_slice(&GGUF_MAGIC);
        buf.extend_from_slice(&3u32.to_le_bytes());
        buf.extend_from_slice(&10u64.to_le_bytes());
        buf.extend_from_slice(&0u64.to_le_bytes()); // 0 KV pairs
        buf.extend_from_slice(&[0u8; 64]);
        file.write_all(&buf).unwrap();

        let mut reader = GgufMappedReader::open(file.path()).expect("Failed to open mapped reader");
        assert!(reader.is_loaded());
        assert_eq!(reader.metadata().version, 3);
        assert_eq!(reader.metadata().tensor_count, 10);

        // Explicitly unload virtual memory map
        reader.unload();
        assert!(!reader.is_loaded());
        assert!(reader.as_slice().is_none());

        // Calling unload multiple times is idempotent
        reader.unload();
        assert!(!reader.is_loaded());
    }

    fn create_synthetic_gguf_file(arch: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        let mut buf = Vec::new();
        buf.extend_from_slice(&GGUF_MAGIC);
        buf.extend_from_slice(&3u32.to_le_bytes()); // Version 3
        buf.extend_from_slice(&10u64.to_le_bytes()); // 10 tensors
        buf.extend_from_slice(&1u64.to_le_bytes()); // 1 KV pair

        let key = b"general.architecture";
        buf.extend_from_slice(&(key.len() as u64).to_le_bytes());
        buf.extend_from_slice(key);
        buf.extend_from_slice(&8u32.to_le_bytes()); // string
        let val = arch.as_bytes();
        buf.extend_from_slice(&(val.len() as u64).to_le_bytes());
        buf.extend_from_slice(val);
        buf.extend_from_slice(&[0u8; 64]);

        file.write_all(&buf).unwrap();
        file
    }

    #[test]
    fn test_gguf_handle_pool_lru_eviction() {
        let pool = BoundedGgufPool::with_capacity(3);
        assert_eq!(pool.capacity(), 3);
        assert_eq!(pool.len(), 0);

        let f1 = create_synthetic_gguf_file("model-1");
        let f2 = create_synthetic_gguf_file("model-2");
        let f3 = create_synthetic_gguf_file("model-3");
        let f4 = create_synthetic_gguf_file("model-4");

        // 1. Open models 1, 2, 3
        let r1 = pool.get_or_open(f1.path()).expect("Failed to open model 1");
        assert!(r1.read().unwrap().is_loaded());
        assert_eq!(pool.len(), 1);

        let r2 = pool.get_or_open(f2.path()).expect("Failed to open model 2");
        assert!(r2.read().unwrap().is_loaded());
        assert_eq!(pool.len(), 2);

        let r3 = pool.get_or_open(f3.path()).expect("Failed to open model 3");
        assert!(r3.read().unwrap().is_loaded());
        assert_eq!(pool.len(), 3);

        // 2. Re-access model 1: this updates model 1's recency timestamp (making model 2 the oldest)
        let _ = pool.get_or_open(f1.path()).expect("Failed to re-access model 1");

        // 3. Open model 4: capacity 3 reached -> triggers automatic eviction of oldest (model 2)
        let r4 = pool.get_or_open(f4.path()).expect("Failed to open model 4");
        assert!(r4.read().unwrap().is_loaded());
        assert_eq!(pool.len(), 3);

        // Model 2 must have suffered .unload() and removal from pool
        assert!(!r2.read().unwrap().is_loaded(), "Victim model 2 must have suffered .unload()");
        assert!(!pool.contains(f2.path()), "Pool must no longer contain model 2");

        // Models 1, 3, 4 must still be loaded in pool
        assert!(r1.read().unwrap().is_loaded(), "Model 1 must remain loaded");
        assert!(r3.read().unwrap().is_loaded(), "Model 3 must remain loaded");
        assert!(pool.contains(f1.path()));
        assert!(pool.contains(f3.path()));
        assert!(pool.contains(f4.path()));
    }

    #[tokio::test]
    async fn test_file_descriptor_exhaustion_prevention() {
        let pool = Arc::new(BoundedGgufPool::new());
        assert_eq!(pool.capacity(), MAX_MAPPED_HANDLES);

        // Create 32 synthetic GGUF model files (double the MAX_MAPPED_HANDLES = 16 ceiling)
        let temp_files: Vec<_> = (0..32)
            .map(|i| create_synthetic_gguf_file(&format!("subagent-model-{}", i)))
            .collect();
        let paths: Arc<Vec<PathBuf>> = Arc::new(temp_files.iter().map(|f| f.path().to_path_buf()).collect());

        // Spawn 20 concurrent subagents making repeated requests across various models
        let mut tasks = Vec::new();
        for subagent_id in 0..20 {
            let pool_clone = Arc::clone(&pool);
            let paths_clone = Arc::clone(&paths);

            tasks.push(tokio::spawn(async move {
                for round in 0..15 {
                    let target_idx = (subagent_id * 7 + round * 3) % paths_clone.len();
                    let target_path = &paths_clone[target_idx];

                    let reader = pool_clone.get_or_open(target_path).expect("Subagent open failed");
                    {
                        let guard = reader.read().expect("Lock poisoned");
                        assert!(guard.is_loaded());
                        assert_eq!(guard.metadata().version, 3);
                    }

                    // Invariant check: pool size and active handles must NEVER exceed MAX_MAPPED_HANDLES
                    assert!(pool_clone.len() <= MAX_MAPPED_HANDLES);
                    assert!(pool_clone.active_handles() <= MAX_MAPPED_HANDLES);
                }
            }));
        }

        for t in tasks {
            t.await.expect("Concurrent subagent task panicked");
        }

        // Final strict boundary assertions
        assert_eq!(pool.len(), MAX_MAPPED_HANDLES);
        assert_eq!(pool.active_handles(), MAX_MAPPED_HANDLES);
    }
}
