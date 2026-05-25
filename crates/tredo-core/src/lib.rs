// tredo-core — Plugin-based provider architecture
// Provides the abstraction layer between core trading logic and AI agents/LLMs.

pub mod plugin_registry;
pub mod provider;
pub mod types;

pub use plugin_registry::{DefaultPluginRegistry, PluginRegistry};
pub use provider::{AgentProvider, LLMProvider};
pub use types::*;

// Re-export for convenience
pub use tredo_types::{ExchangeClient, ExchangeRouter, RiskEngine};
