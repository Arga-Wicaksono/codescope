//! File-system based caching system for codescope.
//!
//! Stores cache entries as individual JSON files in `~/.codescope/cache/`.
//! Each entry tracks creation time, access time, and hit count.
//! Supports TTL-based expiration and size-based cleanup.

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

// ---------------------------------------------------------------------------
// Cache entry
// ---------------------------------------------------------------------------

/// A single cache entry stored on disk as a JSON file.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct CacheEntry {
    key: String,
    value: String, // JSON string of the cached results
    created_at: u64,
    accessed_at: u64,
    hits: u64,
}

impl CacheEntry {
    fn now_epoch_secs() -> u64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

// ---------------------------------------------------------------------------
// Cache stats
// ---------------------------------------------------------------------------

/// Aggregate statistics about the cache.
#[derive(Serialize, Debug)]
pub struct CacheStats {
    pub entries: usize,
    pub total_bytes: usize,
    pub hits: u64,
    pub cache_dir: String,
}

// ---------------------------------------------------------------------------
// Cache manager
// ---------------------------------------------------------------------------

/// Manages a file-system backed cache in `~/.codescope/cache/`.
pub struct CacheManager {
    cache_dir: PathBuf,
    max_age: Duration,
    max_size_mb: usize,
}

impl CacheManager {
    /// Create a new cache manager.
    ///
    /// * `max_age_hours` — entries older than this are considered expired.
    /// * `max_size_mb` — when `cleanup()` is called, oldest entries are evicted
    ///   until the total size on disk is under this limit.
    pub fn new(max_age_hours: u64, max_size_mb: usize) -> Self {
        let cache_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".codescope")
            .join("cache");

        // Ensure the cache directory exists
        let _ = fs::create_dir_all(&cache_dir);

        Self {
            cache_dir,
            max_age: Duration::from_secs(max_age_hours * 3600),
            max_size_mb,
        }
    }

    /// Create a new cache manager with a custom cache directory (useful for
    /// testing).
    #[cfg(test)]
    fn with_cache_dir(cache_dir: PathBuf, max_age_hours: u64, max_size_mb: usize) -> Self {
        let _ = fs::create_dir_all(&cache_dir);
        Self {
            cache_dir,
            max_age: Duration::from_secs(max_age_hours * 3600),
            max_size_mb,
        }
    }

    // ----- key hashing -----------------------------------------------------

    /// Generate a deterministic hex hash string from the given key components.
    /// This does **not** need to be cryptographically secure — it just needs to
    /// be consistent across runs.
    pub fn hash_key(parts: &[&str]) -> String {
        let mut hasher = DefaultHasher::new();
        for part in parts {
            part.hash(&mut hasher);
        }
        format!("{:016x}", hasher.finish())
    }

    /// Convert a hash string to the on-disk file path.
    fn entry_path(&self, key_hash: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.json", key_hash))
    }

    // ----- core operations -------------------------------------------------

    /// Retrieve a cached value by key. Returns `None` if the entry does not
    /// exist or has expired. On a cache hit the `accessed_at` and `hits`
    /// counters are updated on disk.
    pub fn get(&self, key: &str) -> Option<String> {
        let key_hash = Self::hash_key(&[key]);
        let path = self.entry_path(&key_hash);

        let raw = fs::read_to_string(&path).ok()?;

        let mut entry: CacheEntry = serde_json::from_str(&raw).ok()?;

        let now = CacheEntry::now_epoch_secs();

        // TTL check
        let age_secs = now.saturating_sub(entry.created_at);
        if age_secs >= self.max_age.as_secs() {
            // Expired — clean up silently
            let _ = fs::remove_file(&path);
            return None;
        }

        // Update access metadata
        entry.accessed_at = now;
        entry.hits += 1;
        if let Ok(updated) = serde_json::to_string(&entry) {
            let _ = fs::write(&path, updated);
        }

        Some(entry.value)
    }

    /// Store a value in the cache under the given key.
    pub fn set(&self, key: &str, value: &str) -> Result<(), String> {
        let key_hash = Self::hash_key(&[key]);
        let path = self.entry_path(&key_hash);

        let entry = CacheEntry {
            key: key.to_string(),
            value: value.to_string(),
            created_at: CacheEntry::now_epoch_secs(),
            accessed_at: CacheEntry::now_epoch_secs(),
            hits: 0,
        };

        let json = serde_json::to_string(&entry)
            .map_err(|e| format!("Failed to serialize cache entry: {}", e))?;

        fs::write(&path, json)
            .map_err(|e| format!("Failed to write cache entry: {}", e))?;

        Ok(())
    }

    /// Remove a single entry from the cache.
    pub fn invalidate(&self, key: &str) -> Result<(), String> {
        let key_hash = Self::hash_key(&[key]);
        let path = self.entry_path(&key_hash);

        if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| format!("Failed to invalidate cache entry: {}", e))?;
        }

        Ok(())
    }

    /// Remove **all** entries from the cache directory.
    pub fn clear(&self) -> Result<(), String> {
        if !self.cache_dir.exists() {
            return Ok(());
        }

        let entries: Vec<PathBuf> = fs::read_dir(&self.cache_dir)
            .map_err(|e| format!("Failed to read cache directory: {}", e))?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "json"))
            .map(|e| e.path())
            .collect();

        for path in &entries {
            let _ = fs::remove_file(path);
        }

        Ok(())
    }

    /// Return aggregate cache statistics.
    pub fn stats(&self) -> CacheStats {
        let mut entries = 0usize;
        let mut total_bytes = 0usize;
        let mut hits = 0u64;

        if self.cache_dir.exists() {
            if let Ok(dir) = fs::read_dir(&self.cache_dir) {
                for entry in dir.flatten() {
                    if entry.path().extension().map_or(false, |ext| ext == "json") {
                        entries += 1;
                        if let Ok(raw) = fs::read_to_string(entry.path()) {
                            total_bytes += raw.len();
                            if let Ok(ce) = serde_json::from_str::<CacheEntry>(&raw) {
                                hits += ce.hits;
                            }
                        }
                    }
                }
            }
        }

        CacheStats {
            entries,
            total_bytes,
            hits,
            cache_dir: self.cache_dir.to_string_lossy().to_string(),
        }
    }

    /// Remove expired entries and, if total size exceeds `max_size_mb`, evict
    /// the oldest entries (by creation time) until the budget is satisfied.
    ///
    /// Returns the number of entries removed.
    pub fn cleanup(&self) -> Result<usize, String> {
        let mut removed = 0usize;
        let now = CacheEntry::now_epoch_secs();

        // Collect all entries with their metadata
        let mut entries: Vec<(PathBuf, CacheEntry)> = Vec::new();

        if self.cache_dir.exists() {
            if let Ok(dir) = fs::read_dir(&self.cache_dir) {
                for entry in dir.flatten() {
                    if entry.path().extension().map_or(false, |ext| ext == "json") {
                        if let Ok(raw) = fs::read_to_string(entry.path()) {
                            if let Ok(ce) = serde_json::from_str::<CacheEntry>(&raw) {
                                entries.push((entry.path(), ce));
                            }
                        }
                    }
                }
            }
        }

        // 1. Remove expired entries
        let mut live: Vec<(PathBuf, CacheEntry)> = Vec::new();
        for (path, ce) in entries {
            let age_secs = now.saturating_sub(ce.created_at);
            if age_secs >= self.max_age.as_secs() {
                let _ = fs::remove_file(&path);
                removed += 1;
            } else {
                live.push((path, ce));
            }
        }

        // 2. If total size exceeds budget, evict oldest first
        let max_bytes = self.max_size_mb * 1024 * 1024;
        let current_bytes: usize = live
            .iter()
            .filter_map(|(p, _)| fs::metadata(p).ok())
            .map(|m| m.len() as usize)
            .sum();

        if current_bytes > max_bytes {
            // Sort by created_at ascending (oldest first)
            live.sort_by_key(|(_, ce)| ce.created_at);

            let mut freed_bytes = 0usize;
            for (path, _) in &live {
                if current_bytes - freed_bytes <= max_bytes {
                    break;
                }
                if let Ok(meta) = fs::metadata(path) {
                    let _ = fs::remove_file(path);
                    freed_bytes += meta.len() as usize;
                    removed += 1;
                }
            }
        }

        Ok(removed)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a temporary cache manager for testing.
    fn temp_cache(max_age_hours: u64, max_size_mb: usize) -> (tempfile::TempDir, CacheManager) {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = dir.path().join("cache");
        let cm = CacheManager::with_cache_dir(cache_dir, max_age_hours, max_size_mb);
        (dir, cm)
    }

    #[test]
    fn test_set_and_get() {
        let (_dir, cm) = temp_cache(24, 100);
        let key = "content-search:hello-world:.";
        let value = r#"{"results": ["file1.rs", "file2.rs"]}"#;

        cm.set(key, value).unwrap();
        let retrieved = cm.get(key);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), value);
    }

    #[test]
    fn test_get_miss() {
        let (_dir, cm) = temp_cache(24, 100);
        let retrieved = cm.get("nonexistent-key");
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_invalidate() {
        let (_dir, cm) = temp_cache(24, 100);
        let key = "some-search-key";
        cm.set(key, "data").unwrap();
        assert!(cm.get(key).is_some());

        cm.invalidate(key).unwrap();
        assert!(cm.get(key).is_none());
    }

    #[test]
    fn test_clear() {
        let (_dir, cm) = temp_cache(24, 100);
        cm.set("key-a", "val-a").unwrap();
        cm.set("key-b", "val-b").unwrap();
        cm.set("key-c", "val-c").unwrap();

        cm.clear().unwrap();

        let stats = cm.stats();
        assert_eq!(stats.entries, 0);
    }

    #[test]
    fn test_stats() {
        let (_dir, cm) = temp_cache(24, 100);
        cm.set("key-1", r#"{"a":1}"#).unwrap();
        cm.set("key-2", r#"{"b":2}"#).unwrap();

        // Access key-1 a couple of times to bump hits
        let _ = cm.get("key-1");
        let _ = cm.get("key-1");

        let stats = cm.stats();
        assert_eq!(stats.entries, 2);
        assert!(stats.total_bytes > 0);
        assert_eq!(stats.hits, 2);
        assert!(stats.cache_dir.contains("cache"));
    }

    #[test]
    fn test_cleanup_expired() {
        // max_age = 0 hours → everything expires immediately
        let (_dir, cm) = temp_cache(0, 100);
        cm.set("fresh-key", "data").unwrap();
        // With max_age=0, get() also sees the entry as expired, so we
        // verify the file exists on disk directly instead.
        let key_hash = CacheManager::hash_key(&["fresh-key"]);
        let path = cm.entry_path(&key_hash);
        assert!(path.exists());

        // After cleanup the expired entry is removed
        let removed = cm.cleanup().unwrap();
        assert_eq!(removed, 1);
        assert!(!path.exists());
    }

    #[test]
    fn test_hash_key_deterministic() {
        let a = CacheManager::hash_key(&["content-search", "hello", "."]);
        let b = CacheManager::hash_key(&["content-search", "hello", "."]);
        assert_eq!(a, b);
    }

    #[test]
    fn test_hash_key_different_inputs() {
        let a = CacheManager::hash_key(&["content-search", "hello", "."]);
        let b = CacheManager::hash_key(&["content-search", "world", "."]);
        assert_ne!(a, b);
    }

    #[test]
    fn test_hits_increment_on_get() {
        let (_dir, cm) = temp_cache(24, 100);
        cm.set("counter-key", "value").unwrap();

        // First get → hits becomes 1
        let _ = cm.get("counter-key");
        // Second get → hits becomes 2
        let _ = cm.get("counter-key");

        // Read the entry directly from disk to check hits
        let key_hash = CacheManager::hash_key(&["counter-key"]);
        let path = cm.entry_path(&key_hash);
        let raw = fs::read_to_string(&path).unwrap();
        let entry: CacheEntry = serde_json::from_str(&raw).unwrap();
        assert_eq!(entry.hits, 2);
    }

    #[test]
    fn test_cache_entry_serialization_roundtrip() {
        let entry = CacheEntry {
            key: "test-key".to_string(),
            value: r#"{"data": true}"#.to_string(),
            created_at: 1_700_000_000,
            accessed_at: 1_700_000_100,
            hits: 5,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let decoded: CacheEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.key, entry.key);
        assert_eq!(decoded.value, entry.value);
        assert_eq!(decoded.created_at, entry.created_at);
        assert_eq!(decoded.accessed_at, entry.accessed_at);
        assert_eq!(decoded.hits, entry.hits);
    }
}
