//! # SharedMemory — Memory Sharing Layer
//!
//! Provides shared memory between Python Nethra and Rust TREDO agents.
//! Uses Redis hashes for shared state with memory tracking and eviction.
//!
//! ## Key Features
//! - Bidirectional memory sharing (Python ↔ Rust)
//! - Memory pressure tracking and graceful degradation
//! - Automatic eviction of least-important memories
//! - Session-based memory isolation

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::redis_bridge::RedisBridge;

/// A shared memory block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedMemoryBlock {
    pub id: String,
    pub namespace: String,        // "python_nethra", "rust_tredo", "shared"
    pub key: String,
    pub value: serde_json::Value,
    pub owner: String,             // Which side owns this memory
    pub ttl_secs: u64,
    pub size_bytes: u64,
    pub created_at: DateTime<Utc>,
    pub last_written: DateTime<Utc>,
    pub access_count: u64,
}

/// Memory pressure levels
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum MemoryPressure {
    /// Normal operation — no eviction needed
    Normal,
    /// Moderate pressure — start evicting low-importance entries
    Moderate,
    /// High pressure — aggressive eviction
    High,
    /// Critical — emergency eviction of least-used entries
    Critical,
}

impl MemoryPressure {
    pub fn from_usage(current_bytes: u64, max_bytes: u64) -> Self {
        let ratio = if max_bytes > 0 { current_bytes as f64 / max_bytes as f64 } else { 0.0 };
        if ratio > 0.95 {
            MemoryPressure::Critical
        } else if ratio > 0.80 {
            MemoryPressure::High
        } else if ratio > 0.60 {
            MemoryPressure::Moderate
        } else {
            MemoryPressure::Normal
        }
    }
}

/// Shared memory configuration
#[derive(Debug, Clone)]
pub struct SharedMemoryConfig {
    /// Maximum memory usage in bytes before eviction starts
    pub max_memory_bytes: u64,
    /// Maximum entries per namespace
    pub max_entries_per_namespace: usize,
    /// Default TTL for shared entries (seconds)
    pub default_ttl_secs: u64,
    /// How often to check memory pressure (seconds)
    pub pressure_check_interval: u64,
}

impl Default for SharedMemoryConfig {
    fn default() -> Self {
        Self {
            max_memory_bytes: 256 * 1024 * 1024,  // 256 MB
            max_entries_per_namespace: 1000,
            default_ttl_secs: 3600,  // 1 hour
            pressure_check_interval: 60,  // 1 minute
        }
    }
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize)]
pub struct MemoryStats {
    pub total_entries: usize,
    pub python_entries: usize,
    pub rust_entries: usize,
    pub shared_entries: usize,
    pub estimated_bytes: u64,
    pub pressure: MemoryPressure,
    pub python_last_sync: Option<DateTime<Utc>>,
    pub rust_last_sync: Option<DateTime<Utc>>,
}

/// Shared memory layer between Python and Rust
pub struct SharedMemory {
    bridge: Arc<RedisBridge>,
    config: SharedMemoryConfig,
    /// Local tracking of what's been synced
    local_entries: Arc<Mutex<Vec<SharedMemoryBlock>>>,
}

impl SharedMemory {
    pub fn new(bridge: Arc<RedisBridge>, config: Option<SharedMemoryConfig>) -> Self {
        Self {
            bridge,
            config: config.unwrap_or_default(),
            local_entries: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Write a shared memory block (publishes to both Redis and Python via pub/sub)
    pub async fn write(&self, namespace: &str, key: &str, value: serde_json::Value) -> Result<SharedMemoryBlock, String> {
        let block = SharedMemoryBlock {
            id: uuid::Uuid::new_v4().to_string(),
            namespace: namespace.to_string(),
            key: key.to_string(),
            value: value.clone(),
            owner: "rust_tredo".to_string(),
            ttl_secs: self.config.default_ttl_secs,
            size_bytes: serde_json::to_string(&value).unwrap_or_default().len() as u64,
            created_at: Utc::now(),
            last_written: Utc::now(),
            access_count: 0,
        };

        // Store in Redis with TTL
        let json = serde_json::to_string(&block)
            .map_err(|e| format!("Serialize error: {}", e))?;

        let redis_key = format!("memory:{}:{}", namespace, key);
        self.bridge.cache_set(&redis_key, &json, block.ttl_secs).await?;

        // Track locally
        let mut entries = self.local_entries.lock().await;
        entries.push(block.clone());

        // Check memory pressure
        let total_bytes: u64 = entries.iter().map(|e| e.size_bytes).sum();
        let pressure = MemoryPressure::from_usage(total_bytes, self.config.max_memory_bytes);
        if pressure != MemoryPressure::Normal {
            warn!("[SharedMemory] Memory pressure: {:?} ({} bytes / {} max)", 
                  pressure, total_bytes, self.config.max_memory_bytes);
            match pressure {
                MemoryPressure::High | MemoryPressure::Critical => {
                    self.evict().await;
                }
                _ => {}
            }
        }

        // Notify Python Nethra via pub/sub
        self.bridge.publish(
            "nethra:memory",
            &crate::redis_bridge::AgentBusMessage::broadcast(
                "rust_tredo_memory",
                "memory_write",
                serde_json::json!({
                    "namespace": namespace,
                    "key": key,
                    "block_id": block.id,
                    "size_bytes": block.size_bytes,
                    "owner": "rust_tredo",
                }),
            ),
        ).await?;

        Ok(block)
    }

    /// Read a shared memory block
    pub async fn read(&self, namespace: &str, key: &str) -> Result<Option<SharedMemoryBlock>, String> {
        let redis_key = format!("memory:{}:{}", namespace, key);
        let json = self.bridge.cache_get(&redis_key).await?;

        match json {
            Some(data) => {
                let mut block: SharedMemoryBlock = serde_json::from_str(&data)
                    .map_err(|e| format!("Deserialize error: {}", e))?;
                block.access_count += 1;
                // Update access count in Redis
                let updated = serde_json::to_string(&block)
                    .map_err(|e| format!("Serialize error: {}", e))?;
                self.bridge.cache_set(&redis_key, &updated, block.ttl_secs).await?;
                Ok(Some(block))
            }
            None => Ok(None),
        }
    }

    /// Read all blocks in a namespace
    pub async fn read_namespace(&self, namespace: &str) -> Result<Vec<SharedMemoryBlock>, String> {
        let entries = self.bridge.get_all_state().await?;
        let mut blocks = Vec::new();

        for (key, val) in entries {
            if key.starts_with(&format!("memory:{}:", namespace)) {
                if let Ok(block) = serde_json::from_str::<SharedMemoryBlock>(&val) {
                    blocks.push(block);
                }
            }
        }

        Ok(blocks)
    }

    /// Delete a shared memory block
    pub async fn delete(&self, namespace: &str, key: &str) -> Result<(), String> {
        let redis_key = format!("memory:{}:{}", namespace, key);
        // For deletion via Redis, we just let the TTL expire or set a tombstone
        self.bridge.cache_set(&redis_key, "__deleted__", 1).await?;

        let mut entries = self.local_entries.lock().await;
        entries.retain(|e| e.key != key || e.namespace != namespace);

        Ok(())
    }

    /// Evict least-accessed entries under memory pressure
    pub async fn evict(&self) -> usize {
        let mut entries = self.local_entries.lock().await;

        // Sort by access count (ascending) and created_at (oldest first)
        entries.sort_by(|a, b| {
            a.access_count.cmp(&b.access_count)
                .then_with(|| a.created_at.cmp(&b.created_at))
        });

        // Remove bottom 20%
        let total = entries.len();
        let evict_count = (total as f64 * 0.2) as usize;
        if evict_count > 0 && evict_count <= total {
            let evicted: Vec<_> = entries.drain(..evict_count).collect();
            for block in &evicted {
                let redis_key = format!("memory:{}:{}", block.namespace, block.key);
                let _ = self.bridge.cache_set(&redis_key, "__evicted__", 1).await;
            }
            info!("[SharedMemory] Evicted {} entries under memory pressure", evicted.len());
            evicted.len()
        } else {
            0
        }
    }

    /// Get memory statistics
    pub async fn stats(&self) -> MemoryStats {
        let entries = self.local_entries.lock().await;
        let total_bytes: u64 = entries.iter().map(|e| e.size_bytes).sum();

        MemoryStats {
            total_entries: entries.len(),
            python_entries: entries.iter().filter(|e| e.owner == "python_nethra").count(),
            rust_entries: entries.iter().filter(|e| e.owner == "rust_tredo").count(),
            shared_entries: entries.iter().filter(|e| e.namespace == "shared").count(),
            estimated_bytes: total_bytes,
            pressure: MemoryPressure::from_usage(total_bytes, self.config.max_memory_bytes),
            python_last_sync: None,
            rust_last_sync: Some(Utc::now()),
        }
    }
}
