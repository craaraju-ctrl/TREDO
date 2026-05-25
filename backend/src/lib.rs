pub mod routes;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc;

pub use routes::{router, AppState};
use tokio::sync::Mutex;
use tredo_autotrader::{AutoTradingConfig, AutoTradingLoop};
use tredo_bridge::{
    AgentRegistry, CacheConfig, HierarchicalRAG, RAGConfig, RedisBridge, SharedMemory,
    SharedMemoryConfig, TieredCache,
};
use tredo_core::{AgentProvider, DefaultPluginRegistry, LLMProvider, PluginRegistry};
use tredo_data::YahooFinanceProvider;
use tredo_exchange::ExchangeAdapters;
use tredo_execution::{ExecutionEngine, StateCache};
use tredo_intelligence::{IntelligencePool, OllamaLLM};
use tredo_journal::TradeJournal;
use tredo_learning::{LearningConfig, LearningEngine};
use tredo_mcp::McpState;
use tredo_stream::StreamRegistry;
use tredo_types::{ExecutionCommand, RiskEngine, TantraCommand};

pub async fn start_server() {
    tracing_subscriber::fmt::init();
    println!("🚀 TREDO Production Orchestrator v3 starting...");
    println!("   Features: Plugin architecture, WebSocket streaming, Self-learning engine, Sub-agent framework, Tiered KV cache, Redis bridge (Python↔Rust), Hot-swappable providers");

    // Channel capacity
    let (execution_tx, execution_rx) = mpsc::channel::<ExecutionCommand>(1000);
    let (tantra_tx, _tantra_rx) = mpsc::channel::<TantraCommand>(100);

    // Shared lock-free cache
    let cache = StateCache::new();

    // ── Stream Registry (WebSocket broadcast) ─────────────────────────────
    let stream_registry = Arc::new(StreamRegistry::new());
    stream_registry
        .global()
        .alert("info", "TREDO v3 orchestrator initializing");

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
        println!(
            "[TREDO] ⚠️ Redis bridge connection failed: {} (Python-Rust bridge disabled)",
            e
        );
    } else {
        println!("[TREDO] ✅ Redis bridge connected (Python Nethra ↔ Rust TREDO)");
    }

    let agent_registry = Arc::new(AgentRegistry::new(redis_bridge.clone(), "rust_tredo"));

    // Register Rust sub-agents
    let rust_reg = tredo_bridge::AgentRegistration {
        agent_id: "rust_tredo".to_string(),
        agent_type: tredo_bridge::AgentType::RustTREDO,
        display_name: "TREDO Trading Engine".to_string(),
        description: "Rust-based automated trading engine with plugin provider architecture"
            .to_string(),
        capabilities: vec![
            tredo_bridge::AgentCapability::MarketAnalysis,
            tredo_bridge::AgentCapability::TradeExecution,
            tredo_bridge::AgentCapability::RiskAssessment,
            tredo_bridge::AgentCapability::Intelligence,
            tredo_bridge::AgentCapability::SelfLearning,
        ],
        channels: vec![
            "nethra:global".to_string(),
            "nethra:trade_signals".to_string(),
            "nethra:analysis".to_string(),
            "nethra:agent:rust_tredo".to_string(),
        ],
        weight: 1.0,
        status: tredo_bridge::AgentStatus::Active,
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
    println!("[TREDO] Tiered KV cache initialized (L1: in-memory, L2: Redis, shared with Python)");

    // ── Hierarchical RAG DB ───────────────────────────────────────────────
    let rag_db = Arc::new(HierarchicalRAG::new(
        redis_bridge.clone(),
        Some(RAGConfig::default()),
    ));
    println!("[TREDO] Hierarchical RAG initialized (L1: Redis, L2: SQLite, L3: reserved for PG)");

    // ── Shared Memory Layer ───────────────────────────────────────────────
    let shared_memory = Arc::new(SharedMemory::new(
        redis_bridge.clone(),
        Some(SharedMemoryConfig::default()),
    ));
    println!("[TREDO] Shared memory layer initialized (256MB max, auto-eviction on pressure)");

    // ═══════════════════════════════════════════════════════════════════════
    //  Plugin Architecture — Dependency Injection
    // ═══════════════════════════════════════════════════════════════════════
    //
    //  The entire intelligence layer is now pluggable via tredo_core traits:
    //
    //    AgentProvider ─► SkilledAgent (Rust native, 30+ skills)
    //                    NethraBridgeAgent (Python Nethra via Redis)
    //                    GemiCloudAgent (future)
    //
    //    LLMProvider   ─► GeminiLLM (Google Gemini API)
    //                    OpenRouterLLM (future: many models)
    //                    LocalLLM (future: llama.cpp/Ollama)
    //
    //  Providers are registered in the PluginRegistry and can be swapped
    //  at runtime via the /api/providers/* routes without restarting.

    println!("[TREDO] ── Plugin Architecture ────────────────────────────────");

    // 1. Create global native Rust skills evaluator
    let skills_evaluator = Arc::new(tredo_skills::NethraAgent::new());

    // 2. Nethra Swarm Proxy is now handled via the unified SwarmAgentProvider (see below)
    println!("[TREDO]   ✅ Agent Provider: Nethra Swarm Proxy (hot-swappable container)");

    // 3. Create Rust-native agent provider (SkilledAgent wrapping NethraAgent)
    let skilled_agent = Arc::new(tredo_skills::SkilledAgent::new(skills_evaluator.clone()));
    let skilled_agent_arc: Arc<dyn AgentProvider> = skilled_agent.clone();
    println!(
        "[TREDO]   ✅ Agent Provider: skilled (Rust native, 30+ technical/risk/portfolio skills)"
    );

    // 4. Create LLM providers (OllamaLLM running local nemetron:4b, GeminiLLM for risk & reasoning)
    let ollama_llm: Arc<dyn LLMProvider> = Arc::new(OllamaLLM::new(
        Some("http://localhost:11434".to_string()),
        Some("nemetron:4b".to_string()),
    ));
    println!(
        "[TREDO]   ✅ LLM Provider: {} ({})",
        ollama_llm.provider_name(),
        ollama_llm.model_name()
    );

    let gemini_llm: Arc<dyn LLMProvider> = Arc::new(tredo_intelligence::GeminiLLM::new(None, None));
    println!(
        "[TREDO]   ✅ LLM Provider: {} ({})",
        gemini_llm.provider_name(),
        gemini_llm.model_name()
    );

    // 5. Create native Rust Multi-Agent Swarm Provider
    // Pass local_llm (Ollama) and cloud_llm (Gemini) so execution-related bots run locally and Risk Assessor uses Gemini.
    let swarm_bots = tredo_swarm::BotSwarm::new_tredo_swarm(
        skilled_agent_arc.clone(),
        ollama_llm.clone(),
        gemini_llm.clone(),
    );
    // Coordinator (Nethra) uses Gemini for high-quality strategic deep reasoning.
    let swarm_coordinator = tredo_swarm::SwarmCoordinator::new(
        skilled_agent_arc.clone(),
        gemini_llm.clone(),
        swarm_bots,
    );
    let swarm_agent = Arc::new(tredo_swarm::SwarmAgentProvider::new(
        swarm_coordinator,
        "swarm",
    ));
    let swarm_agent_arc: Arc<dyn AgentProvider> = swarm_agent.clone();
    println!("[TREDO]   ✅ Agent Provider: swarm (Rust native hierarchical multi-agent swarm)");

    // 6. Create DI container (PluginRegistry)
    let registry: Arc<dyn PluginRegistry> = Arc::new(DefaultPluginRegistry::new());
    let _ = registry.register_agent("nethra", swarm_agent_arc.clone());
    let _ = registry.register_agent("nethra-swarm", swarm_agent_arc.clone());
    let _ = registry.register_agent("skilled", skilled_agent_arc.clone());
    let _ = registry.register_agent("swarm", swarm_agent_arc.clone());
    let _ = registry.register_llm("ollama", ollama_llm.clone());
    let _ = registry.register_llm("gemini", gemini_llm.clone());
    println!("[TREDO]   ✅ Plugin Registry initialized with Nethra Swarm, Skilled, local LLM provider, and Gemini cloud LLM provider");

    // 7. Create the IntelligencePool (orchestrator)
    // Uses the native Rust multi-agent swarm by default
    let intelligence = Arc::new(IntelligencePool::new(
        swarm_agent_arc.clone(),
        ollama_llm.clone(),
        registry.clone(),
        skills_evaluator.clone(),
        3, // max 3 concurrent LLM requests
        Some(tiered_cache.clone()),
    ));
    println!("[TREDO]   ✅ IntelligencePool orchestrated with active swarm agent + local skills evaluator + Ollama LLM");
    println!("[TREDO] ────────────────────────────────────────────────────────");

    // Start exchange adapters (Binance + KuCoin market feeds)
    let adapters = ExchangeAdapters::new(execution_tx.clone());
    tokio::spawn(async move { adapters.start().await });

    // Start TANTRA coworker safety service
    use tredo_tantra::TantraService;
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
    println!("[TREDO] Self-learning engine initialized");

    // ── Auto-Trading System ───────────────────────────────────────────────

    let data_provider = Arc::new(YahooFinanceProvider::new());

    let journal_path =
        std::env::var("TREDO_JOURNAL_PATH").unwrap_or_else(|_| "tredo_trades.db".to_string());
    let journal = Arc::new(Mutex::new(
        TradeJournal::new(&journal_path).expect("Failed to open trade journal database"),
    ));
    println!("[TREDO] Trade journal initialized at {}", journal_path);

    // Auto-trading loop uses Swarm Agent Provider (via AgentProvider trait)
    let auto_trade_config = AutoTradingConfig::default();
    let auto_trader = Arc::new(AutoTradingLoop::new(
        auto_trade_config,
        data_provider.clone(),
        swarm_agent_arc.clone(), // uses Arc<dyn AgentProvider>
        journal.clone(),
        learning_engine.clone(),
    ));

    // Start the auto-trading loop as a background task
    let auto_trader_clone = auto_trader.clone();
    tokio::spawn(async move {
        auto_trader_clone.run().await;
    });

    println!("[TREDO] Auto-trading loop with pluggable agent provider started (initially paused — enable via API)");

    // Broadcast initial state via WebSocket
    stream_registry.global().alert(
        "info",
        "Auto-trading system initialized. All providers registered.",
    );

    // Subscribe to trade signals from Python Nethra
    let _ = agent_registry.subscribe_global().await;

    // Build Axum state and routes with all new components
    // ── MCP State ───────────────────────────────────────────────────────
    let mcp_state = McpState {
        intelligence: intelligence.clone(),
        data_provider: data_provider.clone(),
        journal: journal.clone(),
        auto_trader: auto_trader.clone(),
        learning_engine: learning_engine.clone(),
        tantra: tantra.clone(),
        stream_registry: stream_registry.clone(),
        redis_bridge: redis_bridge.clone(),
        agent_registry: agent_registry.clone(),
        tiered_cache: tiered_cache.clone(),
        shared_memory: shared_memory.clone(),
        registry: registry.clone(),
        rag_db: rag_db.clone(),
    };

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
        mcp_state,
    };
    let app = router(app_state);

    println!("✅ TREDO server v3 listening on http://0.0.0.0:8080");
    println!("   WebSocket: ws://0.0.0.0:8080/ws");
    println!("   Provider API: /api/providers/* — swap agents/LLMs at runtime");
    println!("   Inspired by Nethra Trismegistus — adaptive, self-learning market intelligence");
    let listener = TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind TCP listener on 0.0.0.0:8080");

    // Graceful shutdown: wait for SIGINT (Ctrl+C) or SIGTERM
    let graceful = async {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!("\n[TREDO] ⏳ SIGINT received — shutting down gracefully...");
            }
            _ = shutdown_signal_sigterm() => {
                println!("\n[TREDO] ⏳ SIGTERM received — shutting down gracefully...");
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
