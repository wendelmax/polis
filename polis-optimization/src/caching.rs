use anyhow::Result;
use dashmap::DashMap;
use lru::LruCache;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Generic cache trait
pub trait Cache<K, V> {
    fn get(&self, key: &K) -> Option<V>;
    fn insert(&mut self, key: K, value: V) -> Option<V>;
    fn remove(&mut self, key: &K) -> Option<V>;
    fn clear(&mut self);
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}

/// LRU cache implementation
pub struct LruCacheWrapper<K, V> {
    cache: LruCache<K, V>,
}

impl<K: Hash + Eq + Clone, V: Clone> LruCacheWrapper<K, V> {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: LruCache::new(capacity),
        }
    }
}

impl<K: Hash + Eq + Clone, V: Clone> Cache<K, V> for LruCacheWrapper<K, V> {
    fn get(&self, key: &K) -> Option<V> {
        self.cache.peek(key).cloned()
    }

    fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.cache.put(key, value)
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        self.cache.pop(key)
    }

    fn clear(&mut self) {
        self.cache.clear();
    }

    fn len(&self) -> usize {
        self.cache.len()
    }

    fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

/// Time-based cache entry
#[derive(Debug, Clone)]
pub struct CacheEntry<V> {
    value: V,
    created_at: Instant,
    ttl: Duration,
}

impl<V> CacheEntry<V> {
    pub fn new(value: V, ttl: Duration) -> Self {
        Self {
            value,
            created_at: Instant::now(),
            ttl,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }

    pub fn value(&self) -> &V {
        &self.value
    }

    pub fn into_value(self) -> V {
        self.value
    }
}

/// TTL-based cache
pub struct TtlCache<K, V> {
    cache: HashMap<K, CacheEntry<V>>,
    default_ttl: Duration,
}

impl<K: Hash + Eq + Clone, V: Clone> TtlCache<K, V> {
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            cache: HashMap::new(),
            default_ttl,
        }
    }

    pub fn with_capacity(capacity: usize, default_ttl: Duration) -> Self {
        Self {
            cache: HashMap::with_capacity(capacity),
            default_ttl,
        }
    }

    pub fn insert_with_ttl(&mut self, key: K, value: V, ttl: Duration) -> Option<V> {
        let entry = CacheEntry::new(value, ttl);
        self.cache.insert(key, entry).map(|e| e.into_value())
    }

    pub fn cleanup_expired(&mut self) {
        self.cache.retain(|_, entry| !entry.is_expired());
    }

    pub fn get_ttl_remaining(&self, key: &K) -> Option<Duration> {
        self.cache.get(key).and_then(|entry| {
            let elapsed = entry.created_at.elapsed();
            if elapsed < entry.ttl {
                Some(entry.ttl - elapsed)
            } else {
                None
            }
        })
    }
}

impl<K: Hash + Eq + Clone, V: Clone> Cache<K, V> for TtlCache<K, V> {
    fn get(&self, key: &K) -> Option<V> {
        self.cache.get(key).and_then(|entry| {
            if entry.is_expired() {
                None
            } else {
                Some(entry.value().clone())
            }
        })
    }

    fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.insert_with_ttl(key, value, self.default_ttl)
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        self.cache.remove(key).map(|e| e.into_value())
    }

    fn clear(&mut self) {
        self.cache.clear();
    }

    fn len(&self) -> usize {
        self.cache.len()
    }

    fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

/// Multi-level cache
pub struct MultiLevelCache<K, V> {
    l1_cache: Arc<RwLock<LruCacheWrapper<K, V>>>,
    l2_cache: Arc<RwLock<TtlCache<K, V>>>,
    l3_cache: Arc<DashMap<K, V>>,
}

impl<K: Hash + Eq + Clone + Send + Sync, V: Clone + Send + Sync> MultiLevelCache<K, V> {
    pub fn new(l1_capacity: usize, l2_ttl: Duration) -> Self {
        Self {
            l1_cache: Arc::new(RwLock::new(LruCacheWrapper::new(l1_capacity))),
            l2_cache: Arc::new(RwLock::new(TtlCache::new(l2_ttl))),
            l3_cache: Arc::new(DashMap::new()),
        }
    }

    pub async fn get(&self, key: &K) -> Option<V> {
        // Try L1 cache first
        if let Some(value) = self.l1_cache.read().await.get(key) {
            return Some(value);
        }

        // Try L2 cache
        if let Some(value) = self.l2_cache.read().await.get(key) {
            // Promote to L1
            if let Ok(mut l1) = self.l1_cache.try_write() {
                l1.insert(key.clone(), value.clone());
            }
            return Some(value);
        }

        // Try L3 cache
        if let Some(value) = self.l3_cache.get(key) {
            let value = value.clone();
            // Promote to L2 and L1
            if let Ok(mut l2) = self.l2_cache.try_write() {
                l2.insert(key.clone(), value.clone());
            }
            if let Ok(mut l1) = self.l1_cache.try_write() {
                l1.insert(key.clone(), value.clone());
            }
            return Some(value);
        }

        None
    }

    pub async fn insert(&self, key: K, value: V) -> Option<V> {
        let mut old_value = None;

        // Insert into all levels
        if let Ok(mut l1) = self.l1_cache.try_write() {
            old_value = l1.insert(key.clone(), value.clone());
        }

        if let Ok(mut l2) = self.l2_cache.try_write() {
            l2.insert(key.clone(), value.clone());
        }

        self.l3_cache.insert(key, value);

        old_value
    }

    pub async fn remove(&self, key: &K) -> Option<V> {
        let mut old_value = None;

        // Remove from all levels
        if let Ok(mut l1) = self.l1_cache.try_write() {
            old_value = l1.remove(key);
        }

        if let Ok(mut l2) = self.l2_cache.try_write() {
            l2.remove(key);
        }

        self.l3_cache.remove(key);

        old_value
    }

    pub async fn clear(&self) {
        if let Ok(mut l1) = self.l1_cache.try_write() {
            l1.clear();
        }

        if let Ok(mut l2) = self.l2_cache.try_write() {
            l2.clear();
        }

        self.l3_cache.clear();
    }

    pub async fn cleanup(&self) {
        if let Ok(mut l2) = self.l2_cache.try_write() {
            l2.cleanup_expired();
        }
    }

    pub async fn stats(&self) -> CacheStats {
        let l1_len = self.l1_cache.read().await.len();
        let l2_len = self.l2_cache.read().await.len();
        let l3_len = self.l3_cache.len();

        CacheStats {
            l1_entries: l1_len,
            l2_entries: l2_len,
            l3_entries: l3_len,
            total_entries: l1_len + l2_len + l3_len,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub l1_entries: usize,
    pub l2_entries: usize,
    pub l3_entries: usize,
    pub total_entries: usize,
}

/// Cache manager for different types of data
pub struct CacheManager {
    container_cache: MultiLevelCache<String, polis_core::types::Container>,
    image_cache: MultiLevelCache<String, polis_core::types::Image>,
    config_cache: MultiLevelCache<String, polis_core::PolisConfig>,
    stats_cache: MultiLevelCache<String, serde_json::Value>,
}

impl CacheManager {
    pub fn new() -> Self {
        Self {
            container_cache: MultiLevelCache::new(1000, Duration::from_secs(300)), // 5 minutes
            image_cache: MultiLevelCache::new(500, Duration::from_secs(600)),      // 10 minutes
            config_cache: MultiLevelCache::new(100, Duration::from_secs(3600)),    // 1 hour
            stats_cache: MultiLevelCache::new(200, Duration::from_secs(60)),       // 1 minute
        }
    }

    pub async fn get_container(&self, id: &str) -> Option<polis_core::types::Container> {
        self.container_cache.get(id).await
    }

    pub async fn set_container(&self, id: String, container: polis_core::types::Container) {
        self.container_cache.insert(id, container).await;
    }

    pub async fn get_image(&self, id: &str) -> Option<polis_core::types::Image> {
        self.image_cache.get(id).await
    }

    pub async fn set_image(&self, id: String, image: polis_core::types::Image) {
        self.image_cache.insert(id, image).await;
    }

    pub async fn get_config(&self, key: &str) -> Option<polis_core::PolisConfig> {
        self.config_cache.get(key).await
    }

    pub async fn set_config(&self, key: String, config: polis_core::PolisConfig) {
        self.config_cache.insert(key, config).await;
    }

    pub async fn get_stats(&self, key: &str) -> Option<serde_json::Value> {
        self.stats_cache.get(key).await
    }

    pub async fn set_stats(&self, key: String, stats: serde_json::Value) {
        self.stats_cache.insert(key, stats).await;
    }

    pub async fn cleanup_all(&self) {
        self.container_cache.cleanup().await;
        self.image_cache.cleanup().await;
        self.config_cache.cleanup().await;
        self.stats_cache.cleanup().await;
    }

    pub async fn get_all_stats(&self) -> HashMap<String, CacheStats> {
        let mut stats = HashMap::new();
        stats.insert("containers".to_string(), self.container_cache.stats().await);
        stats.insert("images".to_string(), self.image_cache.stats().await);
        stats.insert("configs".to_string(), self.config_cache.stats().await);
        stats.insert("stats".to_string(), self.stats_cache.stats().await);
        stats
    }
}

/// Cache warming strategies
pub struct CacheWarmer {
    cache_manager: Arc<CacheManager>,
}

impl CacheWarmer {
    pub fn new(cache_manager: Arc<CacheManager>) -> Self {
        Self { cache_manager }
    }

    pub async fn warm_container_cache(&self, container_ids: Vec<String>) -> Result<()> {
        // This would fetch containers from the runtime and cache them
        for id in container_ids {
            // Simulate fetching container
            let container = polis_core::types::Container {
                id: polis_core::types::ContainerId::new(),
                name: format!("container-{}", id),
                image: polis_core::types::ImageId::new("alpine", "latest"),
                status: polis_core::types::ContainerStatus::Running,
                created_at: chrono::Utc::now(),
                started_at: Some(chrono::Utc::now()),
                finished_at: None,
                exit_code: None,
                command: vec!["echo".to_string(), "hello".to_string()],
                working_dir: std::path::PathBuf::from("/"),
                environment: std::collections::HashMap::new(),
                labels: std::collections::HashMap::new(),
                resource_limits: polis_core::types::ResourceLimits::default(),
                network_mode: polis_core::types::NetworkMode::Bridge,
                ports: vec![],
                volumes: vec![],
            };
            self.cache_manager.set_container(id, container).await;
        }
        Ok(())
    }

    pub async fn warm_image_cache(&self, image_ids: Vec<String>) -> Result<()> {
        // This would fetch images from the image manager and cache them
        for id in image_ids {
            // Simulate fetching image
            let image = polis_core::types::Image {
                id: polis_core::types::ImageId::new("alpine", "latest"),
                name: format!("image-{}", id),
                tag: "latest".to_string(),
                digest: format!("sha256:{}", id),
                size: 1024 * 1024, // 1MB
                created_at: chrono::Utc::now(),
                architecture: "amd64".to_string(),
                os: "linux".to_string(),
                layers: vec![],
                config: polis_core::types::ImageConfig::default(),
            };
            self.cache_manager.set_image(id, image).await;
        }
        Ok(())
    }

    pub async fn warm_all_caches(&self) -> Result<()> {
        // Warm all caches with frequently accessed data
        self.warm_container_cache(vec!["container1".to_string(), "container2".to_string()])
            .await?;
        self.warm_image_cache(vec!["image1".to_string(), "image2".to_string()])
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_lru_cache() {
        let mut cache = LruCacheWrapper::new(2);

        cache.insert("key1", "value1");
        cache.insert("key2", "value2");
        cache.insert("key3", "value3"); // This should evict key1

        assert_eq!(cache.get(&"key1"), None);
        assert_eq!(cache.get(&"key2"), Some("value2"));
        assert_eq!(cache.get(&"key3"), Some("value3"));
    }

    #[test]
    fn test_ttl_cache() {
        let mut cache = TtlCache::new(Duration::from_millis(100));

        cache.insert("key1", "value1");
        assert_eq!(cache.get(&"key1"), Some("value1"));

        std::thread::sleep(Duration::from_millis(150));
        assert_eq!(cache.get(&"key1"), None);
    }

    #[tokio::test]
    async fn test_multi_level_cache() {
        let cache = MultiLevelCache::new(2, Duration::from_secs(1));

        cache.insert("key1", "value1").await;
        assert_eq!(cache.get(&"key1").await, Some("value1"));

        cache.remove(&"key1").await;
        assert_eq!(cache.get(&"key1").await, None);
    }

    #[tokio::test]
    async fn test_cache_manager() {
        let manager = CacheManager::new();

        let container = polis_core::types::Container {
            id: polis_core::types::ContainerId::new(),
            name: "test".to_string(),
            image: polis_core::types::ImageId::new("alpine", "latest"),
            status: polis_core::types::ContainerStatus::Running,
            created_at: chrono::Utc::now(),
            started_at: Some(chrono::Utc::now()),
            finished_at: None,
            exit_code: None,
            command: vec!["echo".to_string(), "hello".to_string()],
            working_dir: std::path::PathBuf::from("/"),
            environment: std::collections::HashMap::new(),
            labels: std::collections::HashMap::new(),
            resource_limits: polis_core::types::ResourceLimits::default(),
            network_mode: polis_core::types::NetworkMode::Bridge,
            ports: vec![],
            volumes: vec![],
        };

        manager
            .set_container("test-id".to_string(), container.clone())
            .await;
        assert!(manager.get_container("test-id").await.is_some());
    }
}
