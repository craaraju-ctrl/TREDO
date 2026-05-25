//! # tredo-bridge — Hybrid Python-Rust Bridge Layer
//!
//! Bridges Nethra (Python) and TREDO (Rust) via Redis pub/sub, providing:
//!
//! - **RedisBridge**: Core pub/sub connection manager for agent communication
//! - **HierarchicalRAG**: Three-tier RAG DB (Redis → SQLite → Postgres)
//! - **SharedMemory**: Memory sharing between Python and Rust agents
//! - **AgentRegistry**: Sub-agent registry with pub/sub message routing
//! - **TieredCache**: Two-tier KV cache (in-memory LRU + Redis)
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────────┐        ┌──────────────┐
//! │  Python       │        │  Rust        │
//! │  Nethra Agent │◄──────►│  TREDO Engine │
//! │  (sub-agents) │  Redis │  (sub-agents)│
//! └──────┬───────┘        └──────┬───────┘
//!        │                       │
//!        └─────── Redis ─────────┘
//!                │
//!        ┌───────┴────────┐
//!        │  pub/sub · KV  │
//!        │  shared state  │
//!        └───────┬────────┘
//!                │
//!   ┌────────────┼────────────┐
//!   │ L1: Redis  │ L2: SQLite │ L3: Postgres
//!   │ (hot)      │ (warm)     │ (cold)
//!   └────────────┴────────────┴────────────┘
//! ```

pub mod agent_registry;
pub mod cache;
pub mod memory;
pub mod nethra_bridge_agent;
pub mod rag;
pub mod redis_bridge;

// Re-exports for convenience
pub use agent_registry::*;
pub use cache::*;
pub use memory::*;
pub use nethra_bridge_agent::*;
pub use rag::*;
pub use redis_bridge::*;
