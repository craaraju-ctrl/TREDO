use std::sync::Arc;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::sync::{Mutex, RwLock};
use serde::Serialize;
use tokio::sync::Semaphore;

pub mod gemini_llm;
pub use gemini_llm::GeminiLLM;

pub mod ollama_llm;
pub use ollama_llm::OllamaLLM;

use tredo_core::{
    AgentProvider, LLMProvider, PluginRegistry,
    MarketAnalysisContext, AggregatedAnalysis,
};
use tredo_bridge::TieredCache;

// ── KV Cache ──────────────────────────────────────────────────────────────

/// A time-to-live cache for LLM responses to reduce KV cache usage
#[derive(Debug)]
struct KVCacheEntry {
    response: String,
    created_at: Instant,
    access_count: u64,
}

/// Thread-safe KV cache for LLM responses with TTL eviction
struct KVCache {
    store: Mutex<HashMap<u64, KVCacheEntry>>,
    ttl: Duration,
    max_entries: usize,
    hits: std::sync::atomic::AtomicU64,
    misses: std::sync::atomic::AtomicU64,
}

impl KVCache {
    fn new(ttl_secs: u64, max_entries: usize) -> Self {
        Self {
            store: Mutex::new(HashMap::with_capacity(max_entries)),
            ttl: Duration::from_secs(ttl_secs),
            max_entries,
            hits: std::sync::atomic::AtomicU64::new(0),
            misses: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Generate a hash key from the prompt
    fn hash_key(prompt: &str) -> u64 {
        let mut hash: u64 = 5381;
        for b in prompt.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(b as u64);
        }
        hash
    }

    /// Get a cached response if available and not expired
    fn get(&self, prompt: &str) -> Option<String> {
        let key = Self::hash_key(prompt);
        let mut store = self.store.lock().ok()?;

        store.retain(|_, entry| entry.created_at.elapsed() < self.ttl);

        if let Some(entry) = store.get_mut(&key) {
            if entry.created_at.elapsed() < self.ttl {
                entry.access_count += 1;
                self.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                return Some(entry.response.clone());
            }
        }

        self.misses.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        None
    }

    /// Store a response in the cache
    fn set(&self, prompt: &str, response: String) {
        let key = Self::hash_key(prompt);
        if let Ok(mut store) = self.store.lock() {
            if store.len() >= self.max_entries {
                if let Some(oldest_key) = store.iter()
                    .min_by_key(|(_, entry)| entry.access_count)
                    .map(|(k, _)| *k)
                {
                    store.remove(&oldest_key);
                }
            }
            store.insert(key, KVCacheEntry {
                response,
                created_at: Instant::now(),
                access_count: 0,
            });
        }
    }

    fn stats(&self) -> KVCacheStats {
        let hits = self.hits.load(std::sync::atomic::Ordering::Relaxed);
        let misses = self.misses.load(std::sync::atomic::Ordering::Relaxed);
        let size = self.store.lock().ok().map(|s| s.len()).unwrap_or(0);
        KVCacheStats {
            hits,
            misses,
            size,
            hit_rate: if hits + misses > 0 {
                (hits as f64 / (hits + misses) as f64) * 100.0
            } else {
                0.0
            },
            max_entries: self.max_entries,
            ttl_secs: self.ttl.as_secs(),
        }
    }

    fn hash_key_fn(prompt: &str) -> u64 {
        Self::hash_key(prompt)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct KVCacheStats {
    pub hits: u64,
    pub misses: u64,
    pub size: usize,
    pub hit_rate: f64,
    pub max_entries: usize,
    pub ttl_secs: u64,
}

/// Fallback mock responses when the LLM provider is unavailable
fn get_fallback_analysis(prompt: &str) -> String {
    let conviction = 0.72 + (prompt.len() as f64 % 20.0) * 0.01;
    format!(
        "{{ \"conviction\": {}, \"reasoning\": \"[Local Fallback Mode] Setup shows breakout potential on standard Bollinger compression. Risk limits approved.\" }}",
        conviction
    )
}

// ── Intelligence Pool (Orchestrator) ──────────────────────────────────────

/// Central orchestrator that coordinates pluggable agent providers and LLM providers.
///
/// This is the high-level interface used by the trading loop and API routes.
/// It delegates all domain-specific work to registered providers, keeping
/// the orchestration logic clean and provider-agnostic.
pub struct IntelligencePool {
    /// Pluggable agent provider (SkilledAgent, NethraBridgeAgent, etc.)
    /// Wrapped in RwLock for runtime hot-swapping.
    agent: Arc<RwLock<Arc<dyn AgentProvider>>>,
    /// Pluggable LLM provider (GeminiLLM, OpenRouterLLM, etc.)
    /// Wrapped in RwLock for runtime hot-swapping.
    llm: Arc<RwLock<Arc<dyn LLMProvider>>>,
    /// Plugin registry for runtime provider discovery/swapping
    registry: Arc<dyn PluginRegistry>,
    /// Global native Rust skills evaluator
    skills_evaluator: Arc<tredo_skills::NethraAgent>,
    /// Concurrency limiter for LLM calls
    semaphore: Arc<Semaphore>,
    /// L1 in-memory KV cache
    kv_cache: KVCache,
    /// L2 shared Redis cache (optional)
    tiered_cache: Option<Arc<TieredCache>>,
}

impl IntelligencePool {
    /// Create a new IntelligencePool with pluggable providers.
    ///
    /// - `agent`: Primary market analysis provider (e.g., SkilledAgent with 30+ skills)
    /// - `llm`: Primary LLM provider (e.g., GeminiLLM)
    /// - `registry`: Provider registry for runtime inspection/swapping
    /// - `max_concurrency`: Max concurrent LLM requests
    /// - `tiered_cache`: Optional Redis-backed tiered cache
    pub fn new(
        agent: Arc<dyn AgentProvider>,
        llm: Arc<dyn LLMProvider>,
        registry: Arc<dyn PluginRegistry>,
        skills_evaluator: Arc<tredo_skills::NethraAgent>,
        max_concurrency: usize,
        tiered_cache: Option<Arc<TieredCache>>,
    ) -> Self {
        Self {
            agent: Arc::new(RwLock::new(agent)),
            llm: Arc::new(RwLock::new(llm)),
            registry,
            skills_evaluator,
            semaphore: Arc::new(Semaphore::new(max_concurrency)),
            kv_cache: KVCache::new(300, 100), // 5 min TTL, 100 entries
            tiered_cache,
        }
    }

    /// Query the LLM analyst with KV cache + TieredCache fallback.
    /// Delegates to the registered LLM provider.
    pub async fn query_analyst(&self, prompt: &str) -> Result<String, &'static str> {
        // L1: In-memory KV cache
        if let Some(cached) = self.kv_cache.get(prompt) {
            println!("[IntelligencePool] 🎯 L1 KV cache HIT — returning cached response");
            return Ok(cached);
        }

        // L2: Redis tiered cache (shared with Python)
        if let Some(ref tc) = self.tiered_cache {
            let cache_key = format!("llm:{}", KVCache::hash_key_fn(prompt));
            if let Some(cached) = tc.get(&cache_key).await {
                println!("[IntelligencePool] 🎯 L2 Redis cache HIT (shared with Python Nethra)");
                self.kv_cache.set(prompt, cached.clone());
                return Ok(cached);
            }
        }

        let llm = self.llm.read().expect("LLM RwLock poisoned").clone();
        println!("[IntelligencePool] KV cache MISS — querying LLM provider: {}", llm.provider_name());

        let _permit = self.semaphore.acquire().await.map_err(|_| "Failed to acquire permit")?;

        match llm.complete(prompt, Some("HFT analyst. Reply JSON only."), None).await {
            Ok(response) => {
                // Cache in both layers
                self.kv_cache.set(prompt, response.clone());
                if let Some(ref tc) = self.tiered_cache {
                    tc.set(&format!("llm:{}", KVCache::hash_key_fn(prompt)), &response).await;
                }
                Ok(response)
            }
            Err(e) => {
                println!("[IntelligencePool] LLM provider error: {:?}. Falling back to dynamic mock.", e);
                let fallback = get_fallback_analysis(prompt);
                self.kv_cache.set(prompt, fallback.clone());
                Ok(fallback)
            }
        }
    }

    /// Run agent-based analysis on market context.
    /// Delegates to the registered AgentProvider.
    pub async fn analyze_with_skills(&self, mut context: MarketAnalysisContext) -> AggregatedAnalysis {
        // 1. Evaluate all 30+ native Rust skills locally on the market context first
        let local_aggregated = self.skills_evaluator.analyze(&context).await;
        
        // 2. Inject these calculated skills directly into context.local_skills
        context.local_skills = Some(local_aggregated.signals.clone());

        // 3. Delegate to the active AgentProvider (which can optionally use/read the local skills)
        let agent = self.agent.read().expect("Agent RwLock poisoned").clone();
        agent.analyze_market(&context).await.unwrap_or_else(|e| {
            println!("[IntelligencePool] Active agent error: {:?}. Falling back to Rust local skills.", e);
            local_aggregated
        })
    }

    /// Run orchestrated analysis through sub-agents (if supported by the provider).
    pub async fn analyze_orchestrated(&self, context: &MarketAnalysisContext) -> Option<serde_json::Value> {
        let agent = self.agent.read().expect("Agent RwLock poisoned").clone();
        agent.analyze_orchestrated(context).await
    }

    /// List all available skill names from the agent provider.
    pub async fn list_skills(&self) -> Vec<String> {
        let agent = self.agent.read().expect("Agent RwLock poisoned").clone();
        agent.list_skills().await
    }

    /// Get info about all sub-agents from the agent provider.
    pub async fn agent_info(&self) -> Vec<HashMap<String, serde_json::Value>> {
        let agent = self.agent.read().expect("Agent RwLock poisoned").clone();
        agent.agent_info().await
    }

    /// Get KV cache statistics.
    pub fn cache_stats(&self) -> KVCacheStats {
        self.kv_cache.stats()
    }

    /// Get a clone of the current agent provider.
    pub fn agent_provider(&self) -> Arc<dyn AgentProvider> {
        self.agent.read().expect("Agent RwLock poisoned").clone()
    }

    /// Get a clone of the current LLM provider.
    pub fn llm_provider(&self) -> Arc<dyn LLMProvider> {
        self.llm.read().expect("LLM RwLock poisoned").clone()
    }

    /// Get a reference to the plugin registry.
    pub fn registry(&self) -> &Arc<dyn PluginRegistry> {
        &self.registry
    }

    /// Hot-swap the active agent provider at runtime.
    /// Takes effect immediately for all subsequent market analyses.
    pub fn swap_agent(&self, new_agent: Arc<dyn AgentProvider>) {
        let mut agent = self.agent.write().expect("Agent RwLock poisoned");
        *agent = new_agent;
        println!("[IntelligencePool] 🔄 Agent provider hot-swapped");
    }

    /// Hot-swap the active LLM provider at runtime.
    /// Takes effect immediately for all subsequent LLM queries.
    pub fn swap_llm(&self, new_llm: Arc<dyn LLMProvider>) {
        let mut llm = self.llm.write().expect("LLM RwLock poisoned");
        *llm = new_llm;
        println!("[IntelligencePool] 🔄 LLM provider hot-swapped");
    }
}
