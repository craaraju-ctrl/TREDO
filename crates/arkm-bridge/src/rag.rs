//! # HierarchicalRAG — Three-Tier RAG Database
//!
//! Provides a tiered retrieval-augmented generation (RAG) database:
//!
//! - **L1 (Hot)**: Redis — Frequently accessed memory, recent context, embedding cache
//! - **L2 (Warm)**: SQLite — Session history, full-text search (FTS5), knowledge graph
//! - **L3 (Cold)**: PostgreSQL — Long-term memory, historical data, analytics
//!
//! ## Architecture
//!
//! ```text
//!    Query
//!      │
//!      ▼
//! ┌──────────┐  miss  ┌──────────┐  miss  ┌──────────┐
//! │ L1: Redis │──────►│ L2: SQLite│──────►│ L3: PG   │
//! │ (hot)     │◄──────│ (warm)   │◄──────│ (cold)   │
//! └──────────┘promote └──────────┘promote └──────────┘
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

use crate::redis_bridge::RedisBridge;

/// A single memory entry in the RAG database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub agent_id: String,
    pub key: String,
    pub content: String,
    pub content_type: String,   // "signal", "decision", "insight", "context"
    pub source: String,         // "python_hermes", "rust_arkm", "technical_analyst", etc.
    pub embedding: Vec<f64>,    // Simple embedding vector
    pub importance: f64,        // 0.0 to 1.0 — how important this memory is
    pub access_count: u64,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub ttl_days: u32,
    pub tier: u8,               // 1 = Redis, 2 = SQLite, 3 = Postgres
    pub metadata: serde_json::Value,
}

/// Search result with relevance score
#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub entry: MemoryEntry,
    pub score: f64,          // 0.0 to 1.0 similarity
    pub source_tier: u8,
}

/// RAG configuration
#[derive(Debug, Clone)]
pub struct RAGConfig {
    /// Max entries in L1 (Redis hot cache)
    pub l1_max_entries: usize,
    /// Max entries in L2 (SQLite warm storage)
    pub l2_max_entries: usize,
    /// TTL for L1 entries (seconds)
    pub l1_ttl_secs: u64,
    /// Minimum importance to promote to L1
    pub promote_threshold: f64,
    /// Maximum embeddings to keep per agent
    pub max_embeddings_per_agent: usize,
    /// Auto-prune interval (seconds)
    pub prune_interval_secs: u64,
}

impl Default for RAGConfig {
    fn default() -> Self {
        Self {
            l1_max_entries: 500,
            l2_max_entries: 10_000,
            l1_ttl_secs: 3600,         // 1 hour in Redis
            promote_threshold: 0.6,
            max_embeddings_per_agent: 100,
            prune_interval_secs: 300,   // 5 minutes
        }
    }
}

/// RAG statistics
#[derive(Debug, Clone, Serialize)]
pub struct RAGStats {
    pub l1_count: usize,
    pub l2_count: usize,
    pub l3_count: usize,
    pub total_entries: usize,
    pub queries_total: u64,
    pub l1_hits: u64,
    pub l2_hits: u64,
    pub l3_hits: u64,
    pub avg_query_time_ms: f64,
}

impl MemoryEntry {
    /// Simple cosine similarity between two embedding vectors
    pub fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }
        let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let mag_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let mag_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
        if mag_a == 0.0 || mag_b == 0.0 {
            return 0.0;
        }
        (dot / (mag_a * mag_b)).clamp(0.0, 1.0)
    }

    /// Create a simple hash-based embedding from text content (no ML dependency)
    pub fn compute_embedding(text: &str) -> Vec<f64> {
        // Create a 64-dimension frequency vector from character bigrams
        // This is a simple bag-of-ngrams approach for semantic similarity
        let mut vec = vec![0.0_f64; 64];
        let text_lower = text.to_lowercase();
        let chars: Vec<char> = text_lower.chars().collect();

        for window in chars.windows(2) {
            let bigram = format!("{}{}", window[0], window[1]);
            let hash = bigram.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
            let idx = (hash as usize) % 64;
            vec[idx] += 1.0;
        }

        // Normalize
        let mag: f64 = vec.iter().map(|x| x * x).sum::<f64>().sqrt();
        if mag > 0.0 {
            for v in vec.iter_mut() {
                *v /= mag;
            }
        }

        vec
    }
}

/// The hierarchical RAG database
pub struct HierarchicalRAG {
    bridge: Arc<RedisBridge>,
    config: RAGConfig,
    sqlite_conn: Arc<Mutex<Option<rusqlite::Connection>>>,
    stats: Arc<Mutex<RAGStats>>,
    query_count: std::sync::atomic::AtomicU64,
    l1_hits: std::sync::atomic::AtomicU64,
    l2_hits: std::sync::atomic::AtomicU64,
    l3_hits: std::sync::atomic::AtomicU64,
}

impl HierarchicalRAG {
    /// Create a new RAG database with the given Redis bridge and config
    pub fn new(bridge: Arc<RedisBridge>, config: Option<RAGConfig>) -> Self {
        let sqlite = rusqlite::Connection::open_in_memory()
            .unwrap_or_else(|_| panic!("Failed to open in-memory SQLite for RAG L2"));

        // Initialize FTS5 table for full-text search
        let _ = sqlite.execute_batch(
            "CREATE VIRTUAL TABLE IF NOT EXISTS memory_fts USING fts5(
                id, agent_id, key, content, content_type, source,
                content='memory', content_rowid='rowid'
            );

            CREATE TABLE IF NOT EXISTS memory (
                id TEXT PRIMARY KEY,
                agent_id TEXT NOT NULL,
                key TEXT NOT NULL,
                content TEXT NOT NULL,
                content_type TEXT NOT NULL DEFAULT 'insight',
                source TEXT NOT NULL DEFAULT 'unknown',
                embedding BLOB,
                importance REAL NOT NULL DEFAULT 0.5,
                access_count INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                last_accessed TEXT NOT NULL,
                ttl_days INTEGER NOT NULL DEFAULT 30,
                tier INTEGER NOT NULL DEFAULT 2,
                metadata TEXT NOT NULL DEFAULT '{}'
            );

            CREATE INDEX IF NOT EXISTS idx_memory_agent ON memory(agent_id);
            CREATE INDEX IF NOT EXISTS idx_memory_type ON memory(content_type);
            CREATE INDEX IF NOT EXISTS idx_memory_importance ON memory(importance DESC);
            ",
        );

        let _ = sqlite.execute("PRAGMA journal_mode=WAL", []);

        Self {
            bridge,
            config: config.unwrap_or_default(),
            sqlite_conn: Arc::new(Mutex::new(Some(sqlite))),
            stats: Arc::new(Mutex::new(RAGStats {
                l1_count: 0,
                l2_count: 0,
                l3_count: 0,
                total_entries: 0,
                queries_total: 0,
                l1_hits: 0,
                l2_hits: 0,
                l3_hits: 0,
                avg_query_time_ms: 0.0,
            })),
            query_count: std::sync::atomic::AtomicU64::new(0),
            l1_hits: std::sync::atomic::AtomicU64::new(0),
            l2_hits: std::sync::atomic::AtomicU64::new(0),
            l3_hits: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Store a memory entry across all tiers
    pub async fn store(&self, entry: MemoryEntry) -> Result<(), String> {
        let json = serde_json::to_string(&entry)
            .map_err(|e| format!("Serialize error: {}", e))?;

        // L1: Store in shared state hash (accessible via get_all_state for search)
        if entry.importance >= self.config.promote_threshold {
            self.bridge.set_state(
                &format!("rag:l1:{}", entry.id),
                &json,
            ).await?;
            // Also set TTL via cache_set so it auto-expires
            self.bridge.cache_set(
                &format!("rag:l1:{}", entry.id),
                &json,
                self.config.l1_ttl_secs,
            ).await?;
        }

        // L2: Store in SQLite
        {
            let guard = self.sqlite_conn.lock().await;
            if let Some(conn) = guard.as_ref() {
                let embedding_blob = bincode::serialize(&entry.embedding).unwrap_or_default();
                let _ = conn.execute(
                    "INSERT OR REPLACE INTO memory (id, agent_id, key, content, content_type, source, embedding, importance, access_count, created_at, last_accessed, ttl_days, tier, metadata)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                    rusqlite::params![
                        entry.id, entry.agent_id, entry.key, entry.content,
                        entry.content_type, entry.source, embedding_blob,
                        entry.importance, entry.access_count,
                        entry.created_at.to_rfc3339(), entry.last_accessed.to_rfc3339(),
                        entry.ttl_days, 2, entry.metadata.to_string()
                    ],
                );
            }
        }

        // Publish memory update to Redis (for Python Hermes to consume)
        self.bridge.publish(
            "hermes:memory",
            &crate::redis_bridge::AgentBusMessage::broadcast(
                "rust_arkm_rag",
                "memory_store",
                serde_json::json!({
                    "entry_id": entry.id,
                    "agent_id": entry.agent_id,
                    "key": entry.key,
                    "content_type": entry.content_type,
                    "importance": entry.importance,
                }),
            ),
        ).await?;

        info!("[RAG] Stored entry {} in memory", entry.id);
        Ok(())
    }

    /// Search across all tiers, returning results sorted by relevance
    pub async fn search(&self, query: &str, agent_id: Option<&str>, limit: usize) -> Result<Vec<SearchResult>, String> {
        self.query_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let query_embedding = MemoryEntry::compute_embedding(query);
        let mut results = Vec::new();

        // L1: Search Redis (hot cache)
        // Redis doesn't support vector search natively, so we scan and compute similarity
        {
            let all_l1: Vec<(String, String)> = self.bridge.get_all_state().await
                .unwrap_or_default();

            for (key, val) in all_l1 {
                if key.starts_with("rag:l1:") {
                    if let Ok(entry) = serde_json::from_str::<MemoryEntry>(&val) {
                        if agent_id.is_none() || agent_id.is_some_and(|aid| entry.agent_id == aid) {
                            let similarity = MemoryEntry::cosine_similarity(&query_embedding, &entry.embedding);
                            if similarity > 0.3 {
                                results.push(SearchResult {
                                    entry,
                                    score: similarity,
                                    source_tier: 1,
                                });
                                self.l1_hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            }
                        }
                    }
                }
            }
        }

        // L2: Search SQLite with FTS5 + embedding similarity
        {
            let guard = self.sqlite_conn.lock().await;
            if let Some(conn) = guard.as_ref() {
                // FTS5 search — must query the FTS virtual table, not the base table
                let fts_query = query.split_whitespace()
                    .map(|w| format!("\"{}\"", w.replace('"', "")))
                    .collect::<Vec<_>>()
                    .join(" OR ");

                // Proper FTS5 query: use rowid subquery into memory_fts virtual table
                let mut stmt = conn.prepare(
                    "SELECT m.id, m.agent_id, m.key, m.content, m.content_type, m.source, m.embedding, m.importance, m.access_count, m.created_at, m.last_accessed, m.ttl_days, m.tier, m.metadata
                     FROM memory m
                     WHERE m.agent_id = ?1
                       AND m.rowid IN (SELECT rowid FROM memory_fts WHERE memory_fts MATCH ?2)
                     ORDER BY m.importance DESC LIMIT ?3"
                ).ok();

                if let Some(ref mut stmt) = stmt {
                    let agent = agent_id.unwrap_or("");
                    if let Ok(rows) = stmt.query_map(rusqlite::params![agent, fts_query, limit as i64], |row| {
                        let embedding_blob: Vec<u8> = row.get(6).unwrap_or_default();
                        let embedding: Vec<f64> = bincode::deserialize(&embedding_blob).unwrap_or_default();
                        Ok(MemoryEntry {
                            id: row.get(0)?,
                            agent_id: row.get(1)?,
                            key: row.get(2)?,
                            content: row.get(3)?,
                            content_type: row.get(4)?,
                            source: row.get(5)?,
                            embedding,
                            importance: row.get(7)?,
                            access_count: row.get::<_, i64>(8)? as u64,
                            created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?)
                                .map(|d| d.with_timezone(&chrono::Utc))
                                .unwrap_or_else(|_| chrono::Utc::now()),
                            last_accessed: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(10)?)
                                .map(|d| d.with_timezone(&chrono::Utc))
                                .unwrap_or_else(|_| chrono::Utc::now()),
                            ttl_days: row.get::<_, i32>(11)? as u32,
                            tier: row.get::<_, i32>(12)? as u8,
                            metadata: serde_json::from_str(&row.get::<_, String>(13)?).unwrap_or_default(),
                        })
                    }) {
                        for row in rows.flatten() {
                            let similarity = MemoryEntry::cosine_similarity(&query_embedding, &row.embedding);
                            if similarity > 0.2 {
                                results.push(SearchResult {
                                    entry: row,
                                    score: similarity,
                                    source_tier: 2,
                                });
                                self.l2_hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            }
                        }
                    }
                }
            }
        }

        // Sort by relevance score (descending)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);

        // Update stats
        if let Ok(mut stats) = self.stats.try_lock() {
            stats.l1_hits = self.l1_hits.load(std::sync::atomic::Ordering::Relaxed);
            stats.l2_hits = self.l2_hits.load(std::sync::atomic::Ordering::Relaxed);
            stats.l3_hits = self.l3_hits.load(std::sync::atomic::Ordering::Relaxed);
            stats.queries_total = self.query_count.load(std::sync::atomic::Ordering::Relaxed);
        }

        Ok(results)
    }

    /// Get RAG statistics
    pub async fn stats(&self) -> RAGStats {
        let l2_count = {
            let guard = self.sqlite_conn.lock().await;
            guard.as_ref()
                .and_then(|conn| conn.query_row("SELECT COUNT(*) FROM memory", [], |row| row.get::<_, i64>(0)).ok())
                .unwrap_or(0) as usize
        };

        if let Ok(mut stats) = self.stats.try_lock() {
            stats.l2_count = l2_count;
            stats.total_entries = stats.l1_count + stats.l2_count + stats.l3_count;
            stats.clone()
        } else {
            RAGStats {
                l1_count: 0,
                l2_count,
                l3_count: 0,
                total_entries: l2_count,
                queries_total: 0,
                l1_hits: self.l1_hits.load(std::sync::atomic::Ordering::Relaxed),
                l2_hits: self.l2_hits.load(std::sync::atomic::Ordering::Relaxed),
                l3_hits: self.l3_hits.load(std::sync::atomic::Ordering::Relaxed),
                avg_query_time_ms: 0.0,
            }
        }
    }
}

/// Bincode-compatible serialization for embeddings
mod bincode {
    pub fn serialize(vec: &[f64]) -> Result<Vec<u8>, ()> {
        let mut bytes = Vec::with_capacity(vec.len() * 8);
        for v in vec {
            bytes.extend_from_slice(&v.to_le_bytes());
        }
        Ok(bytes)
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Vec<f64>, ()> {
        if bytes.is_empty() || !bytes.len().is_multiple_of(8) {
            return Ok(Vec::new());
        }
        let count = bytes.len() / 8;
        let mut vec = Vec::with_capacity(count);
        for chunk in bytes.chunks(8) {
            if chunk.len() == 8 {
                let mut arr = [0u8; 8];
                arr.copy_from_slice(chunk);
                vec.push(f64::from_le_bytes(arr));
            }
        }
        Ok(vec)
    }
}
