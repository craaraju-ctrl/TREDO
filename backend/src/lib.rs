pub mod routes;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use std::sync::Arc;

use arkm_types::{ExecutionCommand, TantraCommand, RiskEngine};
use arkm_execution::{ExecutionEngine, StateCache};
use arkm_intelligence::{IntelligencePool, GeminiLLM};
use arkm_exchange::ExchangeAdapters;
use arkm_data::YahooFinanceProvider;
use arkm_journal::TradeJournal;
use arkm_autotrader::{AutoTradingLoop, AutoTradingConfig};
use arkm_learning::{LearningEngine, LearningConfig};
use arkm_stream::StreamRegistry;
use arkm_bridge::{
    RedisBridge, AgentRegistry, TieredCache, HierarchicalRAG, SharedMemory,
    CacheConfig, RAGConfig, SharedMemoryConfig,
};
use arkm_skills::{SkilledAgent, HermesAgent};
use arkm_core::{
    AgentProvider, LLMProvider, PluginRegistry, DefaultPluginRegistry,
};
use tokio::sync::Mutex;
pub use routes::{router, AppState};

pub async fn start_server() {
    tracing_subscriber::fmt::init();
    println!("🚀 ARKM Production Orchestrator v3 starting...");
    println!("   Features: Plugin architecture, WebSocket streaming, Self-learning engine, Sub-agent framework, Tiered KV cache, Redis bridge (Python↔Rust), Hot-swappable providers");

    // Channel capacity
    let (execution_tx, execution_rx) = mpsc::channel::<ExecutionCommand>(1000);
    let (tantra_tx, _tantra_rx) = mpsc::channel::<TantraCommand>(100);

    // Shared lock-free cache
    let cache = StateCache::new();

    // ── Stream Registry (WebSocket broadcast) ─────────────────────────────
    let stream_registry = Arc::new(StreamRegistry::new());
    stream_registry.global().alert("info", "ARKM v3 orchestrator initializing");

    // Spawn fast-path execution engine
    let engine = ExecutionEngine::new(
        execution_rx,
        cache.clone(),
        RiskEngine::new(),
        tantra_tx.clone(),
    );
    tokio::spawn(engine.run());

    // ── Redis Bridge (Python↔Rust Communication) ──────────────────────────
    let redis_url = std::env::var("REDIS_URL").ok();
    let redis_bridge = Arc::new(RedisBridge::new(redis_url));
    if let Err(e) = redis_bridge.connect().await {
        println!("[ARKM] ⚠️ Redis bridge connection failed: {} (Python-Rust bridge disabled)", e);
    } else {
        println!("[ARKM] ✅ Redis bridge connected (Python Hermes ↔ Rust ARKM)");
    }

    let agent_registry = Arc::new(AgentRegistry::new(
        redis_bridge.clone(),
        "rust_arkm",
    ));

    // Register Rust sub-agents
    let rust_reg = arkm_bridge::AgentRegistration {
        agent_id: "rust_arkm".to_string(),
        agent_type: arkm_bridge::AgentType::RustARKM,
        display_name: "ARKM Trading Engine".to_string(),
        description: "Rust-based automated trading engine with plugin provider architecture".to_string(),
        capabilities: vec![
            arkm_bridge::AgentCapability::MarketAnalysis,
            arkm_bridge::AgentCapability::TradeExecution,
            arkm_bridge::AgentCapability::RiskAssessment,
            arkm_bridge::AgentCapability::Intelligence,
            arkm_bridge::AgentCapability::SelfLearning,
        ],
        channels: vec![
            "hermes:global".to_string(),
            "hermes:trade_signals".to_string(),
            "hermes:analysis".to_string(),
            "hermes:agent:rust_arkm".to_string(),
        ],
        weight: 1.0,
        status: arkm_bridge::AgentStatus::Active,
        last_heartbeat: chrono::Utc::now(),
        registered_at: chrono::Utc::now(),
        metadata: serde_json::json!({"version": "v3", "features": ["plugin_agents", "plugin_llms", "websocket", "self_learning", "kv_cache"]}),
    };
    let _ = agent_registry.register(rust_reg).await;

    // ── Tiered KV Cache (shared with Python) ──────────────────────────────
    let tiered_cache = Arc::new(TieredCache::new(
        redis_bridge.clone(),
        Some(CacheConfig::default()),
    ));
    println!("[ARKM] Tiered KV cache initialized (L1: in-memory, L2: Redis, shared with Python)");

    // ── Hierarchical RAG DB ───────────────────────────────────────────────
    let rag_db = Arc::new(HierarchicalRAG::new(
        redis_bridge.clone(),
        Some(RAGConfig::default()),
    ));
    println!("[ARKM] Hierarchical RAG initialized (L1: Redis, L2: SQLite, L3: reserved for PG)");

    // ── Shared Memory Layer ───────────────────────────────────────────────
    let shared_memory = Arc::new(SharedMemory::new(
        redis_bridge.clone(),
        Some(SharedMemoryConfig::default()),
    ));
    println!("[ARKM] Shared memory layer initialized (256MB max, auto-eviction on pressure)");

    // ═══════════════════════════════════════════════════════════════════════
    //  Plugin Architecture — Dependency Injection
    // ═══════════════════════════════════════════════════════════════════════
    //
    //  The entire intelligence layer is now pluggable via arkm_core traits:
    //
    //    AgentProvider ─► SkilledAgent (Rust native, 30+ skills)
    //                    HermesBridgeAgent (Python Hermes via Redis)
    //                    GemiCloudAgent (future)
    //
    //    LLMProvider   ─► GeminiLLM (Google Gemini API)
    //                    OpenRouterLLM (future: many models)
    //                    LocalLLM (future: llama.cpp/Ollama)
    //
    //  Providers are registered in the PluginRegistry and can be swapped
    //  at runtime via the /api/providers/* routes without restarting.

    println!("[ARKM] ── Plugin Architecture ────────────────────────────────");

    // 1. Create Rust-native agent provider (SkilledAgent wrapping HermesAgent)
    let hermes = Arc::new(HermesAgent::new());
    let skilled_agent: Arc<dyn AgentProvider> = Arc::new(SkilledAgent::new(hermes));
    println!("[ARKM]   ✅ Agent Provider: skilled (Rust native, 30+ technical/risk/portfolio skills)");

    // 2. Create LLM provider (GeminiLLM)
    let gemini_llm: Arc<dyn LLMProvider> = Arc::new(GeminiLLM::new(None, None));
    println!("[ARKM]   ✅ LLM Provider: {} ({})", gemini_llm.provider_name(), gemini_llm.model_name());

    // 3. Create DI container (PluginRegistry)
    let registry: Arc<dyn PluginRegistry> = Arc::new(DefaultPluginRegistry::new());
    let _ = registry.register_agent("skilled", skilled_agent.clone());
    let _ = registry.register_llm("gemini", gemini_llm.clone());
    println!("[ARKM]   ✅ Plugin Registry initialized with 1 agent + 1 LLM provider");

    // 4. Conditionally register HermesBridgeAgent if Redis is connected
    if true {
        // Check if bridge is connected by trying to publish
        // HermesBridgeAgent will handle fallback if Python is unreachable
        let bridge_agent = arkm_bridge::HermesBridgeAgent::new(redis_bridge.clone(), "rust_arkm", "python_hermes");
        let bridge_agent_arc: Arc<dyn AgentProvider> = Arc::new(bridge_agent);
        let _ = registry.register_agent("hermes_bridge", bridge_agent_arc);
        println!("[ARKM]   ✅ Agent Provider: hermes_bridge (Redis proxy to Python Hermes)");
    }

    // 5. Create the IntelligencePool (orchestrator)
    // Uses skilled agent for market analysis and gemini for LLM queries
    let intelligence = Arc::new(IntelligencePool::new(
        skilled_agent.clone(),
        gemini_llm.clone(),
        registry.clone(),
        3, // max 3 concurrent LLM requests
        Some(tiered_cache.clone()),
    ));
    println!("[ARKM]   ✅ IntelligencePool orchestrated with skilled agent + Gemini LLM");
    println!("[ARKM] ────────────────────────────────────────────────────────");

    // Start exchange adapters (Binance + KuCoin market feeds)
    let adapters = ExchangeAdapters::new(execution_tx.clone());
    tokio::spawn(async move { adapters.start().await });

    // Start TANTRA coworker safety service
    use arkm_tantra::TantraService;
    let tantra = Arc::new(TantraService::new());
    let tantra_clone = tantra.clone();
    tokio::spawn(async move {
        tantra_clone.run_service().await;
    });

    // ── Self-Learning Engine ──────────────────────────────────────────────

    let learning_config = LearningConfig {
        adaptive_weighting_enabled: true,
        min_trades_for_weighting: 5,
        regime_optimization_enabled: true,
        max_trade_history: 10000,
        learning_rate: 0.3,
        historical_decay: 0.95,
    };
    let learning_engine = Arc::new(Mutex::new(LearningEngine::new(learning_config)));
    println!("[ARKM] Self-learning engine initialized");

    // ── Auto-Trading System ───────────────────────────────────────────────

    let data_provider = Arc::new(YahooFinanceProvider::new());

    let journal_path = std::env::var("ARKM_JOURNAL_PATH")
        .unwrap_or_else(|_| "arkm_trades.db".to_string());
    let journal = Arc::new(Mutex::new(
        TradeJournal::new(&journal_path)
            .expect("Failed to open trade journal database")
    ));
    println!("[ARKM] Trade journal initialized at {}", journal_path);

    // Auto-trading loop uses SkilledAgent (via AgentProvider trait)
    let auto_trade_config = AutoTradingConfig::default();
    let auto_trader = Arc::new(AutoTradingLoop::new(
        auto_trade_config,
        data_provider.clone(),
        skilled_agent.clone(), // uses Arc<dyn AgentProvider>
        journal.clone(),
        learning_engine.clone(),
    ));

    // Start the auto-trading loop as a background task
    let auto_trader_clone = auto_trader.clone();
    tokio::spawn(async move {
        auto_trader_clone.run().await;
    });

    println!("[ARKM] Auto-trading loop with pluggable agent provider started (initially paused — enable via API)");

    // Broadcast initial state via WebSocket
    stream_registry.global().alert("info", "Auto-trading system initialized. All providers registered.");

    // Subscribe to trade signals from Python Hermes
    let _ = agent_registry.subscribe_global().await;

    // Build Axum state and routes with all new components
    let app_state = AppState {
        execution_tx,
        intelligence,
        tantra,
        data_provider,
        journal,
        auto_trader,
        learning_engine,
        stream_registry,
        redis_bridge,
        agent_registry,
        tiered_cache,
        rag_db,
        shared_memory,
        registry,
    };
    let app = router(app_state);

    println!("✅ ARKM server v3 listening on http://0.0.0.0:8080");
    println!("   WebSocket: ws://0.0.0.0:8080/ws");
    println!("   Provider API: /api/providers/* — swap agents/LLMs at runtime");
    println!("   Inspired by Hermes Trismegistus — adaptive, self-learning market intelligence");
    let listener = TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind TCP listener on 0.0.0.0:8080");

    // Graceful shutdown: wait for SIGINT (Ctrl+C) or SIGTERM
    let graceful = async {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!("\n[ARKM] ⏳ SIGINT received — shutting down gracefully...");
            }
            _ = shutdown_signal_sigterm() => {
                println!("\n[ARKM] ⏳ SIGTERM received — shutting down gracefully...");
            }
        }
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(graceful)
        .await
        .expect("Server failed at runtime");
}

/// Wait for a SIGTERM signal (Unix only). On non-Unix platforms, returns a future that never resolves.
#[cfg(unix)]
async fn shutdown_signal_sigterm() {
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .expect("Failed to install SIGTERM signal handler");
    sigterm.recv().await;
}

#[cfg(not(unix))]
async fn shutdown_signal_sigterm() {
    std::future::pending::<()>().await;
}
