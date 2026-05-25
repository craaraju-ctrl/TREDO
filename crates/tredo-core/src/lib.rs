// tredo-core — Plugin-based provider architecture
// Provides the abstraction layer between core trading logic and AI agents/LLMs.

pub mod types;
pub mod provider;
pub mod plugin_registry;

pub use types::*;
pub use provider::{AgentProvider, LLMProvider};
pub use plugin_registry::{PluginRegistry, DefaultPluginRegistry};

// Re-export for convenience
pub use tredo_types::{RiskEngine, ExchangeRouter, ExchangeClient};
