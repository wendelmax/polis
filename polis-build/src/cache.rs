use crate::{BuildError, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

/// Build cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub id: String,
    pub instruction: String,
    pub content_hash: String,
    pub created_at: SystemTime,
    pub size: u64,
    pub layer_id: Option<String>,
}

/// Build cache manager
#[derive(Debug)]
pub struct BuildCache {
    pub cache_dir: PathBuf,
    pub entries: HashMap<String, CacheEntry>,
    pub max_size: u64,
    pub current_size: u64,
}

impl BuildCache {
    /// Create a new build cache
    pub fn new(cache_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&cache_dir)
            .map_err(|e| BuildError::Io(e))?;

        let mut cache = Self {
            cache_dir,
            entries: HashMap::new(),
            max_size: 10 * 1024 * 1024 * 1024, // 10GB default
            current_size: 0,
        };

        cache.load_cache()?;
        Ok(cache)
    }

    /// Load existing cache entries
    fn load_cache(&mut self) -> Result<()> {
        let cache_file = self.cache_dir.join("cache.json");
        
        if cache_file.exists() {
            let content = std::fs::read_to_string(&cache_file)
                .map_err(|e| BuildError::Io(e))?;
            
            let entries: HashMap<String, CacheEntry> = serde_json::from_str(&content)
                .map_err(|e| BuildError::Parse(format!("Failed to parse cache: {}", e)))?;
            
            self.entries = entries;
            self.current_size = self.entries.values().map(|e| e.size).sum();
        }

        Ok(())
    }

    /// Save cache entries to disk
    fn save_cache(&self) -> Result<()> {
        let cache_file = self.cache_dir.join("cache.json");
        let content = serde_json::to_string_pretty(&self.entries)
            .map_err(|e| BuildError::Parse(format!("Failed to serialize cache: {}", e)))?;
        
        std::fs::write(&cache_file, content)
            .map_err(|e| BuildError::Io(e))?;
        
        Ok(())
    }

    /// Generate a content hash for an instruction
    pub fn generate_content_hash(&self, instruction: &str, context: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(instruction.as_bytes());
        hasher.update(context);
        format!("{:x}", hasher.finalize())
    }

    /// Check if a cache entry exists
    pub fn has_entry(&self, content_hash: &str) -> bool {
        self.entries.contains_key(content_hash)
    }

    /// Get a cache entry
    pub fn get_entry(&self, content_hash: &str) -> Option<&CacheEntry> {
        self.entries.get(content_hash)
    }

    /// Add a new cache entry
    pub fn add_entry(&mut self, instruction: String, content_hash: String, layer_id: Option<String>, size: u64) -> Result<()> {
        // Check if we need to evict old entries
        while self.current_size + size > self.max_size && !self.entries.is_empty() {
            self.evict_oldest()?;
        }

        let entry = CacheEntry {
            id: uuid::Uuid::new_v4().to_string(),
            instruction,
            content_hash: content_hash.clone(),
            created_at: SystemTime::now(),
            size,
            layer_id,
        };

        self.entries.insert(content_hash, entry);
        self.current_size += size;
        
        self.save_cache()?;
        Ok(())
    }

    /// Evict the oldest cache entry
    fn evict_oldest(&mut self) -> Result<()> {
        if let Some((oldest_key, oldest_entry)) = self.entries.iter()
            .min_by_key(|(_, entry)| entry.created_at) {
            
            let key = oldest_key.clone();
            let size = oldest_entry.size;
            
            // Remove the layer file if it exists
            if let Some(layer_id) = &oldest_entry.layer_id {
                let layer_path = self.cache_dir.join(format!("{}.tar", layer_id));
                let _ = std::fs::remove_file(layer_path);
            }
            
            self.entries.remove(&key);
            self.current_size -= size;
        }
        
        Ok(())
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStats {
        CacheStats {
            total_entries: self.entries.len(),
            total_size: self.current_size,
            max_size: self.max_size,
            hit_rate: 0.0, // Would need to track hits/misses
        }
    }

    /// Clear all cache entries
    pub fn clear(&mut self) -> Result<()> {
        // Remove all layer files
        for entry in self.entries.values() {
            if let Some(layer_id) = &entry.layer_id {
                let layer_path = self.cache_dir.join(format!("{}.tar", layer_id));
                let _ = std::fs::remove_file(layer_path);
            }
        }

        self.entries.clear();
        self.current_size = 0;
        self.save_cache()?;
        Ok(())
    }

    /// Get cache directory
    pub fn get_cache_dir(&self) -> &PathBuf {
        &self.cache_dir
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_size: u64,
    pub max_size: u64,
    pub hit_rate: f64,
}
