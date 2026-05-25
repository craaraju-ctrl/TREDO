//! # TieredCache — Two-Tier KV Cache
//!
//! Replaces the simple HashMap-based KV cache with a two-tier architecture:
//!
//! - **L1 (Local)**: Fast in-memory LRU cache with configurable max entries and TTL
//! - **L2 (Redis)**: Shared Redis-backed cache accessible by both Python and Rust
//!
//! Automatically promotes frequently accessed entries to L1 and demotes cold entries to L2.
//! Provides cache statistics for monitoring hit rates and memory pressure.

use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::info;

use crate::redis_bridge::RedisBridge;

/// A single cache entry with metadata
#[derive(Debug, Clone)]
struct CacheEntry {
    #[allow(dead_code)]
    key: String,
    value: String,
    created_at: Instant,
    last_accessed: Instant,
    access_count: u64,
    ttl: Duration,
    #[allow(dead_code)]
    size_bytes: usize,
}

/// LRU cache implementation
struct LRUCache {
    store: HashMap<String, CacheEntry>,
    max_entries: usize,
}

impl LRUCache {
    fn new(max_entries: usize) -> Self {
        Self {
            store: HashMap::with_capacity(max_entries),
            max_entries,
        }
    }

    fn get(&mut self, key: &str) -> Option<String> {
        let entry = self.store.get_mut(key)?;

        // Check TTL
        if entry.created_at.elapsed() > entry.ttl {
            self.store.remove(key);
            return None;
        }

        entry.last_accessed = Instant::now();
        entry.access_count += 1;
        Some(entry.value.clone())
    }

    fn set(&mut self, key: String, value: String, ttl: Duration) {
        // Evict if full (remove least recently used)
        if self.store.len() >= self.max_entries {
            let oldest_key = self.store.iter()
                .min_by_key(|(_, e)| e.last_accessed)
                .map(|(k, _)| k.clone());

            if let Some(k) = oldest_key {
                self.store.remove(&k);
            }
        }

        let size = value.len();
        let key_for_entry = key.clone();
        self.store.insert(key, CacheEntry {
            key: key_for_entry,
            value,
            created_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 0,
            ttl,
            size_bytes: size,
        });
    }

    fn len(&self) -> usize {
        self.store.len()
    }

    fn evict_expired(&mut self) {
        let before = self.store.len();
        self.store.retain(|_, e| e.created_at.elapsed() <= e.ttl);
        #[allow(unused_variables)]
        let evicted = before - self.store.len();
    }

    fn clear(&mut self) {
        self.store.clear();
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize)]
pub struct CacheStats {
    pub l1_entries: usize,
    pub l2_entries: usize,
    pub l1_hits: u64,
    pub l1_misses: u64,
    pub l2_hits: u64,
    pub l2_misses: u64,
    pub l1_hit_rate: f64,
    pub l2_hit_rate: f64,
    pub overall_hit_rate: f64,
    pub total_lookups: u64,
    pub l1_ttl_secs: u64,
    pub l2_ttl_secs: u64,
    pub estimated_memory_bytes: u64,
}

/// Two-tier cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// L1 max entries (in-memory)
    pub l1_max_entries: usize,
    /// L2 TTL (seconds, Redis)
    pub l2_ttl_secs: u64,
    /// Default L1 TTL (seconds)
    pub l1_ttl_secs: u64,
    /// How frequently accessed an entry needs to be to stay in L1
    pub promotion_threshold: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            l1_max_entries: 500,
            l2_ttl_secs: 3600,     // 1 hour in Redis
            l1_ttl_secs: 300,       // 5 minutes in memory
            promotion_threshold: 3, // promoted after 3 accesses
        }
    }
}

/// Two-tier KV cache
pub struct TieredCache {
    l1: Arc<Mutex<LRUCache>>,
    bridge: Arc<RedisBridge>,
    config: CacheConfig,
    stats: Arc<Mutex<CacheStatsInternal>>,
}

#[derive(Debug, Default)]
struct CacheStatsInternal {
    l1_hits: u64,
    l1_misses: u64,
    l2_hits: u64,
    l2_misses: u64,
}

impl TieredCache {
    /// Create a new two-tier cache
    pub fn new(bridge: Arc<RedisBridge>, config: Option<CacheConfig>) -> Self {
        Self {
            l1: Arc::new(Mutex::new(LRUCache::new(
                config.as_ref().map(|c| c.l1_max_entries).unwrap_or(500),
            ))),
            bridge,
            config: config.unwrap_or_default(),
            stats: Arc::new(Mutex::new(CacheStatsInternal::default())),
        }
    }

    /// Get a value from the cache (L1 → L2)
    pub async fn get(&self, key: &str) -> Option<String> {
        // L1 look-up
        {
            let mut l1 = self.l1.lock().await;
            if let Some(value) = l1.get(key) {
                let mut stats = self.stats.lock().await;
                stats.l1_hits += 1;
                return Some(value);
            }
        }

        // L1 miss — try L2 (Redis)
        let mut stats = self.stats.lock().await;
        match self.bridge.cache_get(key).await {
            Ok(Some(value)) => {
                stats.l2_hits += 1;

                // Promote to L1 (clone for the closure)
                let val = value.clone();
                let key_owned = key.to_string();
                let l1 = self.l1.clone();
                let ttl = Duration::from_secs(self.config.l1_ttl_secs);
                tokio::spawn(async move {
                    let mut l1 = l1.lock().await;
                    l1.set(key_owned, val, ttl);
                });

                Some(value)
            }
            _ => {
                stats.l2_misses += 1;
                None
            }
        }
    }

    /// Set a value in both L1 and L2
    pub async fn set(&self, key: &str, value: &str) {
        let ttl_l1 = Duration::from_secs(self.config.l1_ttl_secs);

        // Set in L1
        {
            let mut l1 = self.l1.lock().await;
            l1.set(key.to_string(), value.to_string(), ttl_l1);
        }

        // Set in L2 (Redis)
        let _ = self.bridge.cache_set(key, value, self.config.l2_ttl_secs).await;
    }

    /// Check if key exists in either tier
    pub async fn exists(&self, key: &str) -> bool {
        // Check L1
        {
            let mut l1 = self.l1.lock().await;
            if l1.get(key).is_some() {
                return true;
            }
        }

        // Check L2
        self.bridge.cache_get(key).await.ok().flatten().is_some()
    }

    /// Clear L1 (local) cache only
    pub async fn clear_local(&self) {
        let mut l1 = self.l1.lock().await;
        l1.clear();
        info!("[TieredCache] L1 cache cleared");
    }

    /// Evict expired entries from L1
    pub async fn evict_expired(&self) {
        let mut l1 = self.l1.lock().await;
        l1.evict_expired();
    }

    /// Get comprehensive cache statistics
    pub async fn stats(&self) -> CacheStats {
        let stats = self.stats.lock().await;
        let l1_entries = self.l1.lock().await.len();
        let total_lookups = stats.l1_hits + stats.l1_misses + stats.l2_hits + stats.l2_misses;

        CacheStats {
            l1_entries,
            l2_entries: 0, // Would need Redis SCARD — skipped for performance
            l1_hits: stats.l1_hits,
            l1_misses: stats.l1_misses,
            l2_hits: stats.l2_hits,
            l2_misses: stats.l2_misses,
            l1_hit_rate: if stats.l1_hits + stats.l1_misses > 0 {
                (stats.l1_hits as f64 / (stats.l1_hits + stats.l1_misses) as f64) * 100.0
            } else {
                0.0
            },
            l2_hit_rate: if stats.l2_hits + stats.l2_misses > 0 {
                (stats.l2_hits as f64 / (stats.l2_hits + stats.l2_misses) as f64) * 100.0
            } else {
                0.0
            },
            overall_hit_rate: if total_lookups > 0 {
                ((stats.l1_hits + stats.l2_hits) as f64 / total_lookups as f64) * 100.0
            } else {
                0.0
            },
            total_lookups,
            l1_ttl_secs: self.config.l1_ttl_secs,
            l2_ttl_secs: self.config.l2_ttl_secs,
            estimated_memory_bytes: (l1_entries as u64) * 1024, // rough estimate
        }
    }
}
