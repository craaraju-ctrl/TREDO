use async_trait::async_trait;
use futures::stream::BoxStream;
use std::collections::HashMap;
use std::fmt::Debug;

use crate::types::*;

// ═══════════════════════════════════════════════════════════════
//  AgentProvider — pluggable trading intelligence agent
// ═══════════════════════════════════════════════════════════════

/// A pluggable AI agent that provides market analysis and trading intelligence.
///
/// Implementations can be:
/// - `SkilledAgent` (Rust native: runs technical skills locally)
/// - `NethraBridgeAgent` (proxies to Python Nethra via Redis)
/// - Any future agent (AGiXT, custom LLM agent, etc.)
#[async_trait]
pub trait AgentProvider: Debug + Send + Sync {
    /// Unique name for this provider (e.g., "skilled", "nethra", "openai")
    fn provider_name(&self) -> &str;

    /// Analyze market conditions and produce an aggregated analysis
    async fn analyze_market(
        &self,
        context: &MarketAnalysisContext,
    ) -> Result<AggregatedAnalysis, ProviderError>;

    /// Feed back trade outcomes for learning/adaptation
    async fn learn(&self, feedback: &LearningFeedback) -> Result<(), ProviderError> {
        let _ = feedback;
        Ok(())
    }

    /// List available skill names (for introspection/UI)
    async fn list_skills(&self) -> Vec<String> {
        vec![]
    }

    /// Dynamically adjust agent sub-component weight (for learning engine sync)
    async fn update_weight(&self, _name: &str, _weight: f64) {}

    /// Get structured info about sub-agents (for introspection/UI)
    async fn agent_info(&self) -> Vec<HashMap<String, serde_json::Value>> {
        vec![]
    }

    /// Get orchestrated analysis with sub-agent breakdown (optional)
    /// Returns `None` if this provider doesn't support orchestrated analysis.
    async fn analyze_orchestrated(
        &self,
        _context: &MarketAnalysisContext,
    ) -> Option<serde_json::Value> {
        None
    }
}

// ═══════════════════════════════════════════════════════════════
//  LLMProvider — pluggable large language model backend
// ═══════════════════════════════════════════════════════════════

/// A pluggable LLM provider that can be swapped without touching trading logic.
///
/// Implementations:
/// - `GeminiLLM` (Google Gemini via REST API)
/// - `OpenRouterLLM` (future: unified API for many models)
/// - `LocalLLM` (future: llama.cpp, Ollama, etc.)
#[async_trait]
pub trait LLMProvider: Debug + Send + Sync {
    /// Provider name (e.g., "gemini", "openrouter")
    fn provider_name(&self) -> &str;

    /// Model name (e.g., "gemini-2.5-flash", "gpt-4o")
    fn model_name(&self) -> &str;

    /// Send a prompt and get a text completion
    async fn complete(
        &self,
        prompt: &str,
        system: Option<&str>,
        params: Option<LLMParams>,
    ) -> Result<String, ProviderError>;

    /// Stream a completion token-by-token
    async fn stream_complete(
        &self,
        prompt: &str,
        system: Option<&str>,
        params: Option<LLMParams>,
    ) -> Result<BoxStream<'static, String>, ProviderError>;

    /// Generate embedding vectors for text
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f64>>, ProviderError>;
}
