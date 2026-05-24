//! # RedisBridge — Core pub/sub connection manager
//!
//! Manages Redis connections for Python↔Rust communication.
//! Provides:
//! - Pub/sub channels for agent-to-agent messaging
//! - Shared state hashes (Python ↔ Rust)
//! - Connection pooling with automatic reconnection
//! - Heartbeat/presence tracking

use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{error, info};

/// How often to send heartbeats
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
/// How long without heartbeat before agent is considered dead
const AGENT_TTL: usize = 90;
/// Default Redis URL
const DEFAULT_REDIS_URL: &str = "redis://127.0.0.1:6379";

/// A single message on the agent bus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBusMessage {
    /// Unique message ID
    pub id: String,
    /// Source agent ID
    pub source: String,
    /// Target agent ID (empty = broadcast)
    pub target: String,
    /// Message type (e.g., "signal", "request", "response", "state_sync")
    pub msg_type: String,
    /// Payload as JSON value
    pub payload: serde_json::Value,
    /// Timestamp (ISO 8601)
    pub timestamp: String,
    /// TTL in seconds (message expires after this)
    pub ttl_secs: u64,
}

impl AgentBusMessage {
    pub fn new(source: &str, target: &str, msg_type: &str, payload: serde_json::Value) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            source: source.to_string(),
            target: target.to_string(),
            msg_type: msg_type.to_string(),
            payload,
            timestamp: chrono::Utc::now().to_rfc3339(),
            ttl_secs: 60,
        }
    }

    /// Create a broadcast message (target = "")
    pub fn broadcast(source: &str, msg_type: &str, payload: serde_json::Value) -> Self {
        Self::new(source, "", msg_type, payload)
    }

    /// Check if this message has expired
    pub fn is_expired(&self) -> bool {
        match chrono::DateTime::parse_from_rfc3339(&self.timestamp) {
            Ok(parsed) => {
                let created = parsed.with_timezone(&chrono::Utc);
                let elapsed = chrono::Utc::now() - created;
                elapsed.num_seconds() > self.ttl_secs as i64
            }
            Err(_) => false,
        }
    }
}

/// Pub/sub channel names
pub struct Channels;
impl Channels {
    /// Global broadcast for all agents
    pub const GLOBAL: &'static str = "hermes:global";
    /// Agent-to-agent direct messages
    pub fn direct(agent_id: &str) -> String {
        format!("hermes:agent:{}", agent_id)
    }
    /// Sub-agent analysis results
    pub const ANALYSIS: &'static str = "hermes:analysis";
    /// Trade signals from Rust to Python
    pub const TRADE_SIGNALS: &'static str = "hermes:trade_signals";
    /// State sync requests/responses
    pub const STATE_SYNC: &'static str = "hermes:state_sync";
    /// Heartbeat channel
    pub const HEARTBEAT: &'static str = "hermes:heartbeat";
    /// Memory updates (RAG writes)
    pub const MEMORY_UPDATES: &'static str = "hermes:memory";
}

/// Redis keys for shared state
pub struct StateKeys;
impl StateKeys {
    /// Shared memory store (hash)
    pub const SHARED_MEMORY: &'static str = "hermes:shared_memory";
    /// KV cache namespace
    pub fn cache_key(key: &str) -> String {
        format!("hermes:cache:{}", key)
    }
    /// RAG embedding store
    pub fn rag_key(key: &str) -> String {
        format!("hermes:rag:{}", key)
    }
    /// Per-agent state
    pub fn agent_state(agent_id: &str) -> String {
        format!("hermes:agent_state:{}", agent_id)
    }
}

/// Bridge statistics
#[derive(Debug, Clone, Serialize)]
pub struct BridgeStats {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub connected_agents: u32,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub memory_usage_bytes: u64,
    pub uptime_seconds: u64,
}

/// The core Redis bridge — manages connections, pub/sub, and shared state
pub struct RedisBridge {
    /// Redis connection manager
    conn: Arc<Mutex<Option<ConnectionManager>>>,
    /// Redis URL
    redis_url: String,
    /// Bridge ID (unique instance ID)
    bridge_id: String,
    /// Statistics
    stats: Arc<Mutex<BridgeStats>>,
    /// Started timestamp
    started_at: Instant,
    /// Subscription handles
    subscriptions: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>>,
}

impl RedisBridge {
    /// Create a new Redis bridge with the given URL
    pub fn new(redis_url: Option<String>) -> Self {
        Self {
            conn: Arc::new(Mutex::new(None)),
            redis_url: redis_url.unwrap_or_else(|| DEFAULT_REDIS_URL.to_string()),
            bridge_id: format!("rust-arkm-{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("0")),
            stats: Arc::new(Mutex::new(BridgeStats {
                messages_sent: 0,
                messages_received: 0,
                connected_agents: 0,
                cache_hits: 0,
                cache_misses: 0,
                memory_usage_bytes: 0,
                uptime_seconds: 0,
            })),
            started_at: Instant::now(),
            subscriptions: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get the bridge ID
    pub fn bridge_id(&self) -> &str {
        &self.bridge_id
    }

    /// Connect to Redis and start background tasks
    pub async fn connect(&self) -> Result<(), String> {
        let client = redis::Client::open(self.redis_url.as_str())
            .map_err(|e| format!("Failed to create Redis client: {}", e))?;

        let conn = client
            .get_connection_manager()
            .await
            .map_err(|e| format!("Failed to connect to Redis: {}", e))?;

        let mut guard = self.conn.lock().await;
        *guard = Some(conn);

        info!("[RedisBridge] Connected to Redis at {}", self.redis_url);

        // Start heartbeat task
        let bridge_id = self.bridge_id.clone();
        let conn_arc = self.conn.clone();
        let heartbeat = tokio::spawn(async move {
            loop {
                tokio::time::sleep(HEARTBEAT_INTERVAL).await;
                if let Some(conn) = conn_arc.lock().await.as_mut() {
                    let _: Result<(), _> = conn
                        .set_ex::<_, _, ()>(
                            format!("hermes:heartbeat:{}", bridge_id),
                            chrono::Utc::now().to_rfc3339(),
                            AGENT_TTL as u64,
                        )
                        .await;
                }
            }
        });

        self.subscriptions.lock().await.push(heartbeat);

        Ok(())
    }

    /// Get a usable Redis connection (auto-reconnects if needed)
    async fn get_conn(&self) -> Result<tokio::sync::MutexGuard<'_, Option<ConnectionManager>>, String> {
        {
            let guard = self.conn.lock().await;
            if guard.is_some() {
                return Ok(guard);
            }
        } // drop guard before connect to avoid deadlock
        self.connect().await?;
        let guard = self.conn.lock().await;
        Ok(guard)
    }

    // ── Pub/Sub ──────────────────────────────────────────────────────────

    /// Publish a message to a channel
    pub async fn publish(&self, channel: &str, message: &AgentBusMessage) -> Result<u64, String> {
        let mut guard = self.get_conn().await?;
        let conn = guard.as_mut().ok_or("No Redis connection")?;

        let payload = serde_json::to_string(message)
            .map_err(|e| format!("Serialize error: {}", e))?;

        let count: u64 = conn.publish(channel, payload).await
            .map_err(|e| format!("Publish error: {}", e))?;

        // Track stats
        if let Ok(mut stats) = self.stats.try_lock() {
            stats.messages_sent += 1;
        }

        info!("[RedisBridge] Published to {} ({} subscribers)", channel, count);
        Ok(count)
    }

    /// Subscribe to a channel and process messages with a callback
    pub async fn subscribe<F, Fut>(&self, channel: &str, callback: F) -> Result<(), String>
    where
        F: Fn(AgentBusMessage) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send,
    {
        let client = redis::Client::open(self.redis_url.as_str())
            .map_err(|e| format!("Failed to create Redis client: {}", e))?;

        #[allow(deprecated)]
        let conn = client
            .get_async_connection()
            .await
            .map_err(|e| format!("Failed to get async connection: {}", e))?;

        let mut pubsub = conn.into_pubsub();
        pubsub.subscribe(channel).await
            .map_err(|e| format!("Subscribe error: {}", e))?;

        let channel_name = channel.to_string();
        let stats = self.stats.clone();

        let handle = tokio::spawn(async move {
            use futures::StreamExt;
            info!("[RedisBridge] Subscribed to channel: {}", channel_name);
            let mut stream = pubsub.into_on_message();
            while let Some(msg) = StreamExt::next(&mut stream).await {
                let payload: String = msg.get_payload().unwrap_or_default();
                if let Ok(bus_msg) = serde_json::from_str::<AgentBusMessage>(&payload) {
                    // Update stats (std Mutex, no .await needed inside spawn)
                    if let Ok(mut s) = stats.try_lock() {
                        s.messages_received += 1;
                    }
                    callback(bus_msg).await;
                }
            }
            error!("[RedisBridge] Subscription stream ended for {}", channel_name);
        });

        self.subscriptions.lock().await.push(handle);
        Ok(())
    }

    // ── Shared State ─────────────────────────────────────────────────────

    /// Set a shared state value (persists in Redis hash)
    pub async fn set_state(&self, key: &str, value: &str) -> Result<(), String> {
        let mut guard = self.get_conn().await?;
        let conn = guard.as_mut().ok_or("No Redis connection")?;

        conn.hset::<_, _, _, ()>(StateKeys::SHARED_MEMORY, key, value).await
            .map_err(|e| format!("HSET error: {}", e))?;

        Ok(())
    }

    /// Get a shared state value
    pub async fn get_state(&self, key: &str) -> Result<Option<String>, String> {
        let mut guard = self.get_conn().await?;
        let conn = guard.as_mut().ok_or("No Redis connection")?;

        let val: Option<String> = conn.hget(StateKeys::SHARED_MEMORY, key).await
            .map_err(|e| format!("HGET error: {}", e))?;

        if val.is_some() {
            if let Ok(mut stats) = self.stats.try_lock() {
                stats.cache_hits += 1;
            }
        } else {
            if let Ok(mut stats) = self.stats.try_lock() {
                stats.cache_misses += 1;
            }
        }

        Ok(val)
    }

    /// Get all shared state keys
    pub async fn get_all_state(&self) -> Result<Vec<(String, String)>, String> {
        let mut guard = self.get_conn().await?;
        let conn = guard.as_mut().ok_or("No Redis connection")?;

        let entries: Vec<(String, String)> = conn.hgetall(StateKeys::SHARED_MEMORY).await
            .map_err(|e| format!("HGETALL error: {}", e))?;

        Ok(entries)
    }

    // ── Cache Operations ─────────────────────────────────────────────────

    /// Set a cache entry with TTL
    pub async fn cache_set(&self, key: &str, value: &str, ttl_secs: u64) -> Result<(), String> {
        let mut guard = self.get_conn().await?;
        let conn = guard.as_mut().ok_or("No Redis connection")?;

        conn.set_ex::<_, _, ()>(StateKeys::cache_key(key), value, ttl_secs).await
            .map_err(|e| format!("Cache SET error: {}", e))?;

        Ok(())
    }

    /// Get a cache entry
    pub async fn cache_get(&self, key: &str) -> Result<Option<String>, String> {
        let mut guard = self.get_conn().await?;
        let conn = guard.as_mut().ok_or("No Redis connection")?;

        let val: Option<String> = conn.get(StateKeys::cache_key(key)).await
            .map_err(|e| format!("Cache GET error: {}", e))?;

        if val.is_some() {
            if let Ok(mut stats) = self.stats.try_lock() {
                stats.cache_hits += 1;
            }
        } else {
            if let Ok(mut stats) = self.stats.try_lock() {
                stats.cache_misses += 1;
            }
        }

        Ok(val)
    }

    // ── Heartbeat / Presence ─────────────────────────────────────────────

    /// Get all connected agents (from heartbeats)
    pub async fn get_connected_agents(&self) -> Result<Vec<String>, String> {
        let mut guard = self.get_conn().await?;
        let conn = guard.as_mut().ok_or("No Redis connection")?;

        let keys: Vec<String> = conn.keys("hermes:heartbeat:*").await
            .map_err(|e| format!("KEYS error: {}", e))?;

        let agents: Vec<String> = keys
            .into_iter()
            .map(|k| k.trim_start_matches("hermes:heartbeat:").to_string())
            .collect();

        // Update stats
        if let Ok(mut stats) = self.stats.try_lock() {
            stats.connected_agents = agents.len() as u32;
        }

        Ok(agents)
    }

    /// Get bridge statistics
    pub async fn stats(&self) -> BridgeStats {
        if let Ok(mut stats) = self.stats.try_lock() {
            stats.uptime_seconds = self.started_at.elapsed().as_secs();
            stats.clone()
        } else {
            BridgeStats {
                messages_sent: 0,
                messages_received: 0,
                connected_agents: 0,
                cache_hits: 0,
                cache_misses: 0,
                memory_usage_bytes: 0,
                uptime_seconds: self.started_at.elapsed().as_secs(),
            }
        }
    }

    /// Shutdown all subscriptions
    pub async fn shutdown(&self) {
        if let Ok(subs) = self.subscriptions.try_lock() {
            for handle in subs.iter() {
                handle.abort();
            }
        }
        info!("[RedisBridge] Shutdown complete");
    }
}

impl std::fmt::Debug for RedisBridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisBridge")
            .field("redis_url", &self.redis_url)
            .field("bridge_id", &self.bridge_id)
            .field("started_at", &self.started_at.elapsed().as_secs())
            .finish()
    }
}

impl Default for RedisBridge {
    fn default() -> Self {
        Self::new(None)
    }
}
