/// Simple in-memory caching layer for database queries
/// Reduces database hits for frequently accessed data with configurable TTL
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Cache entry with expiration timestamp
#[derive(Clone)]
struct CacheEntry<T: Clone> {
    value: T,
    expires_at: Instant,
}

impl<T: Clone> CacheEntry<T> {
    fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }
}

/// Query result cache with TTL support
pub struct QueryCache<K, V>
where
    K: std::hash::Hash + Eq + Clone,
    V: Clone,
{
    cache: Arc<Mutex<HashMap<K, CacheEntry<V>>>>,
    ttl: Duration,
}

impl<K, V> QueryCache<K, V>
where
    K: std::hash::Hash + Eq + Clone,
    V: Clone,
{
    /// Create a new cache with specified TTL
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            ttl: Duration::from_secs(ttl_secs),
        }
    }

    /// Get value from cache if it exists and hasn't expired
    pub fn get(&self, key: &K) -> Option<V> {
        let mut cache = self.cache.lock().unwrap();
        if let Some(entry) = cache.get(key) {
            if !entry.is_expired() {
                return Some(entry.value.clone());
            } else {
                // Remove expired entry
                cache.remove(key);
            }
        }
        None
    }

    /// Insert value into cache
    pub fn insert(&self, key: K, value: V) {
        let mut cache = self.cache.lock().unwrap();
        cache.insert(
            key,
            CacheEntry {
                value,
                expires_at: Instant::now() + self.ttl,
            },
        );
    }

    /// Clear all entries from cache
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }

    /// Invalidate specific entry
    pub fn invalidate(&self, key: &K) {
        let mut cache = self.cache.lock().unwrap();
        cache.remove(key);
    }

    /// Get cache size
    pub fn size(&self) -> usize {
        let cache = self.cache.lock().unwrap();
        cache.len()
    }
}

impl<K, V> Clone for QueryCache<K, V>
where
    K: std::hash::Hash + Eq + Clone,
    V: Clone,
{
    fn clone(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache),
            ttl: self.ttl,
        }
    }
}

/// Global cache instances for common queries
pub struct CacheManager {
    pub device_list_cache: QueryCache<String, Vec<String>>, // interface_id -> device IDs
    pub device_count_cache: QueryCache<String, i64>,         // interface_id -> count
    pub alert_rules_cache: QueryCache<String, Vec<String>>,  // "all" -> rule IDs
}

impl CacheManager {
    /// Create cache manager with default TTLs
    pub fn new() -> Self {
        Self {
            device_list_cache: QueryCache::new(30),      // 30 second TTL
            device_count_cache: QueryCache::new(30),     // 30 second TTL
            alert_rules_cache: QueryCache::new(300),     // 5 minute TTL
        }
    }

    /// Invalidate all caches (after mutations)
    pub fn invalidate_all(&self) {
        self.device_list_cache.clear();
        self.device_count_cache.clear();
        self.alert_rules_cache.clear();
    }

    /// Invalidate device-related caches
    pub fn invalidate_devices(&self, interface_id: Option<&str>) {
        if let Some(iface) = interface_id {
            self.device_list_cache.invalidate(&iface.to_string());
            self.device_count_cache.invalidate(&iface.to_string());
        } else {
            self.device_list_cache.clear();
            self.device_count_cache.clear();
        }
    }

    /// Invalidate rule caches
    pub fn invalidate_rules(&self) {
        self.alert_rules_cache.clear();
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_insert_and_retrieve() {
        let cache: QueryCache<String, i32> = QueryCache::new(10);
        cache.insert("key1".to_string(), 42);

        assert_eq!(cache.get(&"key1".to_string()), Some(42));
    }

    #[test]
    fn test_cache_expiration() {
        let cache: QueryCache<String, i32> = QueryCache::new(0); // 0 second TTL
        cache.insert("key1".to_string(), 42);

        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(cache.get(&"key1".to_string()), None);
    }

    #[test]
    fn test_cache_miss() {
        let cache: QueryCache<String, i32> = QueryCache::new(10);
        assert_eq!(cache.get(&"nonexistent".to_string()), None);
    }

    #[test]
    fn test_cache_clear() {
        let cache: QueryCache<String, i32> = QueryCache::new(10);
        cache.insert("key1".to_string(), 1);
        cache.insert("key2".to_string(), 2);

        assert_eq!(cache.size(), 2);
        cache.clear();
        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn test_cache_invalidate() {
        let cache: QueryCache<String, i32> = QueryCache::new(10);
        cache.insert("key1".to_string(), 1);
        cache.insert("key2".to_string(), 2);

        cache.invalidate(&"key1".to_string());

        assert_eq!(cache.get(&"key1".to_string()), None);
        assert_eq!(cache.get(&"key2".to_string()), Some(2));
    }

    #[test]
    fn test_cache_manager() {
        let manager = CacheManager::new();

        manager
            .device_list_cache
            .insert("eth0".to_string(), vec!["dev1".to_string()]);
        manager.device_count_cache.insert("eth0".to_string(), 1);

        assert_eq!(
            manager.device_list_cache.get(&"eth0".to_string()),
            Some(vec!["dev1".to_string()])
        );
        assert_eq!(manager.device_count_cache.get(&"eth0".to_string()), Some(1));

        manager.invalidate_devices(Some("eth0"));

        assert_eq!(manager.device_list_cache.get(&"eth0".to_string()), None);
        assert_eq!(manager.device_count_cache.get(&"eth0".to_string()), None);
    }

    #[test]
    fn test_cache_clone() {
        let cache: QueryCache<String, i32> = QueryCache::new(10);
        cache.insert("key1".to_string(), 42);

        let cloned = cache.clone();
        // Cloned cache shares the same backing store
        assert_eq!(cloned.get(&"key1".to_string()), Some(42));
    }
}
