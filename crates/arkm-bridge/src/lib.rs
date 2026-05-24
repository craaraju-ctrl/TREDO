//! # arkm-bridge — Hybrid Python-Rust Bridge Layer
//!
//! Bridges Hermes (Python) and ARKM (Rust) via Redis pub/sub, providing:
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
//! │  Hermes Agent │◄──────►│  ARKM Engine │
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

pub mod redis_bridge;
pub mod rag;
pub mod memory;
pub mod agent_registry;
pub mod cache;
pub mod hermes_bridge_agent;

// Re-exports for convenience
pub use redis_bridge::*;
pub use rag::*;
pub use memory::*;
pub use agent_registry::*;
pub use cache::*;
pub use hermes_bridge_agent::*;
