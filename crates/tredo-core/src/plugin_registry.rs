use crate::provider::{AgentProvider, LLMProvider};
use crate::types::ProviderError;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// ═══════════════════════════════════════════════════════════════
//  PluginRegistry — Dependency Injection Container
// ═══════════════════════════════════════════════════════════════

/// Central registry for all pluggable providers (agents + LLMs).
///
/// This is the DI container for the entire application. All providers
/// are registered at startup and can be resolved dynamically at runtime,
/// enabling easy swapping without code changes.
pub trait PluginRegistry: Send + Sync {
    /// Register an agent provider under a given name
    fn register_agent(
        &self,
        name: &str,
        agent: Arc<dyn AgentProvider>,
    ) -> Result<(), ProviderError>;

    /// Get an agent provider by name
    fn get_agent(&self, name: &str) -> Option<Arc<dyn AgentProvider>>;

    /// Register an LLM provider under a given name
    fn register_llm(
        &self,
        name: &str,
        llm: Arc<dyn LLMProvider>,
    ) -> Result<(), ProviderError>;

    /// Get an LLM provider by name
    fn get_llm(&self, name: &str) -> Option<Arc<dyn LLMProvider>>;

    /// List all registered agent names
    fn list_agents(&self) -> Vec<String>;

    /// List all registered LLM names
    fn list_llms(&self) -> Vec<String>;
}

// ═══════════════════════════════════════════════════════════════
//  DefaultPluginRegistry — RwLock-backed DI Container
// ═══════════════════════════════════════════════════════════════

/// Thread-safe, runtime-mutable implementation of PluginRegistry.
///
/// Uses `RwLock<HashMap>` under the hood for O(1) lookups.
/// Providers can be registered at startup and swapped at runtime
/// via the API without restarting the server.
#[derive(Debug)]
pub struct DefaultPluginRegistry {
    agents: RwLock<HashMap<String, Arc<dyn AgentProvider>>>,
    llms: RwLock<HashMap<String, Arc<dyn LLMProvider>>>,
}

impl DefaultPluginRegistry {
    pub fn new() -> Self {
        Self {
            agents: RwLock::new(HashMap::new()),
            llms: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for DefaultPluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginRegistry for DefaultPluginRegistry {
    fn register_agent(
        &self,
        name: &str,
        agent: Arc<dyn AgentProvider>,
    ) -> Result<(), ProviderError> {
        let mut agents = self.agents.write().expect("Agents RwLock poisoned");
        agents.insert(name.to_string(), agent);
        Ok(())
    }

    fn get_agent(&self, name: &str) -> Option<Arc<dyn AgentProvider>> {
        let agents = self.agents.read().expect("Agents RwLock poisoned");
        agents.get(name).cloned()
    }

    fn register_llm(
        &self,
        name: &str,
        llm: Arc<dyn LLMProvider>,
    ) -> Result<(), ProviderError> {
        let mut llms = self.llms.write().expect("LLMs RwLock poisoned");
        llms.insert(name.to_string(), llm);
        Ok(())
    }

    fn get_llm(&self, name: &str) -> Option<Arc<dyn LLMProvider>> {
        let llms = self.llms.read().expect("LLMs RwLock poisoned");
        llms.get(name).cloned()
    }

    fn list_agents(&self) -> Vec<String> {
        let agents = self.agents.read().expect("Agents RwLock poisoned");
        agents.keys().cloned().collect()
    }

    fn list_llms(&self) -> Vec<String> {
        let llms = self.llms.read().expect("LLMs RwLock poisoned");
        llms.keys().cloned().collect()
    }
}
