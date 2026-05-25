//! TredoModule Integration Test
//!
//! Verifies the full end-to-end flow:
//!   TredoModule frontend API calls → Backend HTTP routes → Database persistence
//!
//! This spins up a real Axum HTTP server with a minimal but functional AppState,
//! exercises the same endpoints that TredoModule calls, and then reads the SQLite
//! journal directly to verify data was persisted correctly.

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::TcpListener;

use tredo_server::{router, AppState};
use tredo_core::{DefaultPluginRegistry, PluginRegistry, AgentProvider, LLMProvider};
use tredo_journal::TradeJournal;
use tredo_autotrader::{AutoTradingLoop, AutoTradingConfig};
use tredo_learning::{LearningEngine, LearningConfig};
use tredo_intelligence::IntelligencePool;
use tredo_data::YahooFinanceProvider;
use tredo_stream::StreamRegistry;
use tredo_skills::{NethraAgent, SkilledAgent};
use tredo_tantra::TantraService;
use tredo_bridge::{
    RedisBridge, AgentRegistry, TieredCache, HierarchicalRAG, SharedMemory,
    CacheConfig, RAGConfig, SharedMemoryConfig,
};
use tredo_mcp::McpState;

/// Build a minimal but functional AppState for integration testing.
///
/// Uses SQLite `:memory:` for the trade journal so every test gets a clean DB.
/// All other dependencies (Redis bridge, agent/LLM providers) are minimally
/// initialized — they exist structurally but won't actually make network calls
/// unless explicitly exercised by the test.
async fn build_test_app_state() -> (AppState, Arc<Mutex<TradeJournal>>, Arc<Mutex<LearningEngine>>, Arc<AutoTradingLoop>) {
    // Set a dummy API key so GeminiLLM doesn't panic on construction.
    // The LLM is never called by any of the routes tested below.
    let _ = std::env::var("GEMINI_API_KEY").unwrap_or_else(|_| {
        std::env::set_var("GEMINI_API_KEY", "test-dummy-key");
        "test-dummy-key".to_string()
    });

    // ── Bootstrap providers ────────────────────────────────────────────
    let registry: Arc<dyn PluginRegistry> = Arc::new(DefaultPluginRegistry::new());

    // Agent + LLM providers (constructed but won't be hit by the routes we test)
    let gemini_llm: Arc<dyn LLMProvider> = Arc::new(
        tredo_intelligence::GeminiLLM::new(None, None),
    );
    let nethra = Arc::new(NethraAgent::new());
    let skilled_agent: Arc<dyn AgentProvider> = Arc::new(
        SkilledAgent::new(nethra.clone()),
    );
    let _ = registry.register_agent("skilled", skilled_agent.clone());
    let _ = registry.register_llm("gemini", gemini_llm.clone());

    // ── Redis bridge (won't connect — no URL) ──────────────────────────
    let redis_bridge = Arc::new(RedisBridge::new(None));
    let tiered_cache = Arc::new(TieredCache::new(
        redis_bridge.clone(),
        Some(CacheConfig::default()),
    ));

    // ── Intelligence pool ──────────────────────────────────────────────
    let intelligence = Arc::new(IntelligencePool::new(
        skilled_agent.clone(),
        gemini_llm.clone(),
        registry.clone(),
        nethra.clone(),
        1, // max concurrent = 1 (won't be queried in these tests)
        Some(tiered_cache.clone()),
    ));

    // ── Trade journal (in-memory SQLite) ───────────────────────────────
    let journal = Arc::new(Mutex::new(
        TradeJournal::new(":memory:").expect("Failed to create in-memory journal"),
    ));

    // ── Learning engine ────────────────────────────────────────────────
    let learning_engine = Arc::new(Mutex::new(
        LearningEngine::new(LearningConfig::default()),
    ));

    // ── Data provider ──────────────────────────────────────────────────
    let data_provider = Arc::new(YahooFinanceProvider::new());

    // ── Auto-trading loop (starts paused) ──────────────────────────────
    let auto_trader_config = AutoTradingConfig::default(); // enabled: false
    let auto_trader = Arc::new(AutoTradingLoop::new(
        auto_trader_config,
        data_provider.clone(),
        skilled_agent.clone(),
        journal.clone(),
        learning_engine.clone(),
    ));

    // ── TANTRA safety service ──────────────────────────────────────────
    let tantra = Arc::new(TantraService::new());

    // ── Stream registry ────────────────────────────────────────────────
    let stream_registry = Arc::new(StreamRegistry::new());

    // ── Bridge components ──────────────────────────────────────────────
    let agent_registry = Arc::new(AgentRegistry::new(
        redis_bridge.clone(),
        "test_tredo",
    ));
    let rag_db = Arc::new(HierarchicalRAG::new(
        redis_bridge.clone(),
        Some(RAGConfig::default()),
    ));
    let shared_memory = Arc::new(SharedMemory::new(
        redis_bridge.clone(),
        Some(SharedMemoryConfig::default()),
    ));

    // ── MCP state ──────────────────────────────────────────────────────
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

    // ── Execution channel ──────────────────────────────────────────────
    let (execution_tx, _execution_rx) = tokio::sync::mpsc::channel(100);

    let app_state = AppState {
        execution_tx,
        intelligence,
        tantra,
        data_provider,
        journal: journal.clone(),
        auto_trader: auto_trader.clone(),
        learning_engine: learning_engine.clone(),
        stream_registry,
        redis_bridge,
        agent_registry,
        tiered_cache,
        rag_db,
        shared_memory,
        registry,
        mcp_state,
    };

    (app_state, journal, learning_engine, auto_trader)
}

/// Helper: spin up the Axum server on a random port and return the base URL.
async fn spawn_test_server(state: AppState) -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to random port");
    let addr = listener.local_addr().unwrap();

    let app = router(state);

    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("Server failed at runtime");
    });

    format!("http://{}", addr)
}

/// Helper: build an HTTP client for sending JSON requests.
fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap()
}

// ═══════════════════════════════════════════════════════════════════════════
//  Tests
// ═══════════════════════════════════════════════════════════════════════════

/// ── 1. Auto-Trading Status ─────────────────────────────────────────────
///
/// The TredoModule calls `GET /api/autotrade/status` on mount to get the
/// initial trading state (enabled, balance, positions, etc.).
#[tokio::test]
async fn test_autotrade_status_returns_initial_state() {
    let (state, _, _, _) = build_test_app_state().await;
    let base_url = spawn_test_server(state).await;

    let resp = client()
        .get(format!("{}/api/autotrade/status", base_url))
        .send()
        .await
        .expect("GET /api/autotrade/status failed");

    assert_eq!(resp.status(), 200, "Expected 200 OK");

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "success", "API should return success");

    let trading = body["trading_state"].as_object()
        .expect("trading_state should be an object");

    // Default AutoTradingConfig: enabled=false, paper_trading=true, BTC-USD, ETH-USD, SOL-USD
    assert_eq!(trading["enabled"], false, "Should start disabled");
    assert_eq!(trading["paper_trading"], true, "Should start in paper mode");
    assert_eq!(trading["balance"], 100_000.0, "Default paper balance should be 100,000");
    assert_eq!(trading["open_positions"], serde_json::Value::Array(vec![]), "No open positions initially");

    let symbols = trading["symbols"].as_array().unwrap();
    assert!(symbols.iter().any(|s| s == "BTC-USD"), "Default symbols should include BTC-USD");
    assert!(symbols.iter().any(|s| s == "ETH-USD"), "Default symbols should include ETH-USD");
}

/// ── 2. Auto-Trading Start/Stop ─────────────────────────────────────────
///
/// The TredoModule calls `POST /api/autotrade/start` and
/// `POST /api/autotrade/stop` to toggle autonomous trading.
#[tokio::test]
async fn test_autotrade_start_then_stop_toggles_state() {
    let (state, _, _, auto_trader) = build_test_app_state().await;
    let base_url = spawn_test_server(state).await;
    let client = client();

    // ── Step 1: Verify initially disabled ──────────────────────────
    let initial_state = auto_trader.get_state().await;
    assert!(!initial_state.enabled, "Should start disabled");

    // ── Step 2: POST /api/autotrade/start ──────────────────────────
    let resp = client
        .post(format!("{}/api/autotrade/start", base_url))
        .send()
        .await
        .expect("POST /api/autotrade/start failed");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "success");
    assert!(body["message"].to_string().to_lowercase().contains("started"));

    // Verify via direct state access
    let after_start = auto_trader.get_state().await;
    assert!(after_start.enabled, "Auto-trader should be enabled after start");

    // ── Step 3: Verify via GET /api/autotrade/status ───────────────
    let resp = client
        .get(format!("{}/api/autotrade/status", base_url))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["trading_state"]["enabled"], true, "Status endpoint should reflect enabled");

    // ── Step 4: POST /api/autotrade/stop ───────────────────────────
    let resp = client
        .post(format!("{}/api/autotrade/stop", base_url))
        .send()
        .await
        .expect("POST /api/autotrade/stop failed");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "success");
    assert!(body["message"].to_string().to_lowercase().contains("stopped"));

    // Verify via direct state access
    let after_stop = auto_trader.get_state().await;
    assert!(!after_stop.enabled, "Auto-trader should be disabled after stop");
}

/// ── 3. Multiple Start/Stop Toggles ────────────────────────────────────
///
/// Verifies that rapid toggling works correctly and state persists.
#[tokio::test]
async fn test_autotrade_multiple_toggles() {
    let (state, _, _, auto_trader) = build_test_app_state().await;
    let base_url = spawn_test_server(state).await;
    let client = client();

    // Toggle: start → stop → start → stop
    for (expected_enabled, action) in [(true, "start"), (false, "stop"), (true, "start"), (false, "stop")] {
        let resp = client
            .post(format!("{}/api/autotrade/{}", base_url, action))
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), 200);

        let state = auto_trader.get_state().await;
        assert_eq!(
            state.enabled, expected_enabled,
            "After '{}', enabled should be {}",
            action, expected_enabled
        );

        // Also verify via API
        let status_resp = client
            .get(format!("{}/api/autotrade/status", base_url))
            .send()
            .await
            .unwrap();

        let body: serde_json::Value = status_resp.json().await.unwrap();
        assert_eq!(
            body["trading_state"]["enabled"], expected_enabled,
            "Status endpoint should reflect state after '{}'", action
        );
    }
}

/// ── 4. Start/Stop Idempotency ──────────────────────────────────────────
///
/// Starting when already started or stopping when already stopped
/// should still return success without errors.
#[tokio::test]
async fn test_autotrade_idempotent_start_stop() {
    let (state, _, _, auto_trader) = build_test_app_state().await;
    let base_url = spawn_test_server(state).await;
    let client = client();

    // Start twice
    let _ = client.post(format!("{}/api/autotrade/start", base_url)).send().await.unwrap();
    let resp2 = client.post(format!("{}/api/autotrade/start", base_url)).send().await.unwrap();
    assert_eq!(resp2.status(), 200);
    assert!(auto_trader.get_state().await.enabled, "Should remain enabled after second start");

    // Stop twice
    let _ = client.post(format!("{}/api/autotrade/stop", base_url)).send().await.unwrap();
    let resp2 = client.post(format!("{}/api/autotrade/stop", base_url)).send().await.unwrap();
    assert_eq!(resp2.status(), 200);
    assert!(!auto_trader.get_state().await.enabled, "Should remain disabled after second stop");
}

/// ── 5. Journal Stats — Empty ──────────────────────────────────────────
///
/// The TredoModule calls `GET /api/journal/stats` for performance stats.
/// With a fresh DB, stats should show zero trades.
#[tokio::test]
async fn test_journal_stats_empty_database() {
    let (state, journal, _, _) = build_test_app_state().await;
    let base_url = spawn_test_server(state).await;

    let resp = client()
        .get(format!("{}/api/journal/stats", base_url))
        .send()
        .await
        .expect("GET /api/journal/stats failed");

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "success");

    let stats = body["stats"].as_object()
        .expect("stats should be an object");

    assert_eq!(stats["total_trades"], 0, "Empty DB should have 0 trades");
    assert_eq!(stats["winning_trades"], 0);
    assert_eq!(stats["losing_trades"], 0);
    assert_eq!(stats["total_pnl"], 0.0);
    assert_eq!(stats["win_rate"].as_f64(), Some(0.0));

    // Also verify direct journal access matches
    let journal_guard = journal.lock().await;
    let direct_stats = journal_guard.get_performance_stats().unwrap();
    assert_eq!(direct_stats.total_trades, 0, "Direct DB query should match API");
}

/// ── 6. Journal Stats — Trades Persist to Database ─────────────────────
///
/// Records trades directly in the journal (as the autotrader would), then
/// verifies that the API returns the correct aggregated stats.
#[tokio::test]
async fn test_journal_stats_reflects_persisted_trades() {
    let (state, journal, _, _) = build_test_app_state().await;
    let base_url = spawn_test_server(state).await;

    // ── Step 1: Simulate the auto-trader recording trades ──────────
    {
        let j = journal.lock().await;

        // Open trade 1: BUY BTC at 60k
        let trade1 = tredo_journal::TradeRecord {
            id: "test-trade-1".to_string(),
            symbol: "BTC-USD".to_string(),
            side: "BUY".to_string(),
            entry_price: 60000.0,
            exit_price: None,
            quantity: 0.5,
            pnl: None,
            pnl_pct: None,
            conviction_at_entry: 0.75,
            entry_reasoning: "Test buy".to_string(),
            exit_reasoning: None,
            market_regime: "trending_bullish".to_string(),
            strategies_used: "integration_test".to_string(),
            open_time: chrono::Utc::now(),
            close_time: None,
            is_open: true,
        };
        j.open_trade(&trade1).unwrap();

        // Record a decision
        let decision = tredo_journal::DecisionRecord {
            id: "test-decision-1".to_string(),
            symbol: "BTC-USD".to_string(),
            timestamp: chrono::Utc::now(),
            overall_conviction: 0.75,
            overall_direction: "BUY".to_string(),
            market_regime: "trending_bullish".to_string(),
            action_taken: "BUY".to_string(),
            reason: "Strong technical signals".to_string(),
            bullish_signals: 8,
            bearish_signals: 2,
            neutral_signals: 1,
        };
        j.record_decision(&decision).unwrap();

        // Close trade 1 with profit
        j.close_trade("test-trade-1", 65000.0, "Take profit at resistance").unwrap();

        // Open trade 2: BUY ETH at 3k
        let trade2 = tredo_journal::TradeRecord {
            id: "test-trade-2".to_string(),
            symbol: "ETH-USD".to_string(),
            side: "BUY".to_string(),
            entry_price: 3000.0,
            exit_price: None,
            quantity: 2.0,
            pnl: None,
            pnl_pct: None,
            conviction_at_entry: 0.6,
            entry_reasoning: "Test buy 2".to_string(),
            exit_reasoning: None,
            market_regime: "ranging".to_string(),
            strategies_used: "integration_test".to_string(),
            open_time: chrono::Utc::now(),
            close_time: None,
            is_open: true,
        };
        j.open_trade(&trade2).unwrap();
    }

    // ── Step 2: Verify via API ─────────────────────────────────────
    let resp = client()
        .get(format!("{}/api/journal/stats", base_url))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "success");

    let stats = body["stats"].as_object().unwrap();

    // 1 closed trade + 1 open trade => 1 closed (in stats)
    assert_eq!(stats["total_trades"], 1, "Only closed trades count in stats");
    assert_eq!(stats["winning_trades"], 1, "BTC trade was profitable");
    assert!(stats["total_pnl"].as_f64().unwrap() > 0.0, "BTC profit should be positive");

    // ── Step 3: Verify decisions are also persisted ────────────────
    let decisions_resp = client()
        .get(format!("{}/api/journal/decisions", base_url))
        .send()
        .await
        .unwrap();

    assert_eq!(decisions_resp.status(), 200);
    let decisions_body: serde_json::Value = decisions_resp.json().await.unwrap();
    assert_eq!(decisions_body["status"], "success");

    let decisions = decisions_body["decisions"].as_array().unwrap();
    assert_eq!(decisions.len(), 1, "Should have 1 decision record");
    assert_eq!(decisions[0]["symbol"], "BTC-USD");
    assert_eq!(decisions[0]["action_taken"], "BUY");
}

/// ── 7. Journal Stats — Multiple Trades P&L Aggregation ────────────────
///
/// Records a mix of winning and losing trades and verifies the aggregated
/// stats (win rate, total P&L, profit factor) are computed correctly.
#[tokio::test]
async fn test_journal_stats_mixed_trades() {
    let (state, journal, _, _) = build_test_app_state().await;
    let base_url = spawn_test_server(state).await;

    // Simulate the auto-trader recording multiple trades
    {
        let j = journal.lock().await;

        let trades = vec![
            ("t1", "BTC-USD", 60000.0, 65000.0, 1.0),   // +$5,000
            ("t2", "ETH-USD", 3000.0,  2800.0,  2.0),   // -$400
            ("t3", "SOL-USD", 140.0,   155.0,   10.0),  // +$150
            ("t4", "BTC-USD", 62000.0, 61000.0, 0.5),   // -$500
            ("t5", "ETH-USD", 2900.0,  3100.0,  1.5),   // +$300
        ];

        for (id, symbol, entry, exit, qty) in &trades {
            j.open_trade(&tredo_journal::TradeRecord {
                id: id.to_string(),
                symbol: symbol.to_string(),
                side: "BUY".to_string(),
                entry_price: *entry,
                exit_price: None,
                quantity: *qty,
                pnl: None,
                pnl_pct: None,
                conviction_at_entry: 0.7,
                entry_reasoning: "Test".to_string(),
                exit_reasoning: None,
                market_regime: "trending_bullish".to_string(),
                strategies_used: "integration_test".to_string(),
                open_time: chrono::Utc::now(),
                close_time: None,
                is_open: true,
            }).unwrap();

            j.close_trade(id, *exit, "Test exit").unwrap();
        }
    }

    // Verify via API
    let resp = client()
        .get(format!("{}/api/journal/stats", base_url))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let stats = body["stats"].as_object().unwrap();

    assert_eq!(stats["total_trades"], 5, "All 5 trades should be counted");
    assert_eq!(stats["winning_trades"], 3, "3 winning trades");
    assert_eq!(stats["losing_trades"], 2, "2 losing trades");

    // Win rate = (3/5) * 100 = 60%
    assert!((stats["win_rate"].as_f64().unwrap() - 60.0).abs() < 0.01,
        "Win rate should be 60%");

    // Total P&L = 5000 + (-400) + 150 + (-500) + 300 = $4,550
    let expected_pnl = 5000.0 - 400.0 + 150.0 - 500.0 + 300.0;
    assert!((stats["total_pnl"].as_f64().unwrap() - expected_pnl).abs() < 0.01,
        "Total P&L should be ${}", expected_pnl);
}

/// ── 8. Health Check ──────────────────────────────────────────────────
///
/// Verifies the health endpoint responds correctly.
#[tokio::test]
async fn test_health_check() {
    let (state, _, _, _) = build_test_app_state().await;
    let base_url = spawn_test_server(state).await;

    let resp = client()
        .get(format!("{}/api/health", base_url))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "operational");
    assert!(body["engine"].to_string().contains("TREDO"));
}

/// ── 9. Journal Decisions — DB Persistence ────────────────────────────
///
/// Records decisions and verifies they're queryable via the API and
/// directly from the database.
#[tokio::test]
async fn test_journal_decisions_persistence() {
    let (state, journal, _, _) = build_test_app_state().await;
    let base_url = spawn_test_server(state).await;

    // Record 3 decisions with different symbols
    {
        let j = journal.lock().await;
        for i in 0..3 {
            let decision = tredo_journal::DecisionRecord {
                id: format!("decision-{}", i),
                symbol: format!("SYMBOL-{}", i),
                timestamp: chrono::Utc::now(),
                overall_conviction: 0.5 + (i as f64 * 0.1),
                overall_direction: if i % 2 == 0 { "BUY".to_string() } else { "SELL".to_string() },
                market_regime: "ranging".to_string(),
                action_taken: if i % 2 == 0 { "BUY".to_string() } else { "SKIP".to_string() },
                reason: format!("Test decision {}", i),
                bullish_signals: 3 + i,
                bearish_signals: 2,
                neutral_signals: 1,
            };
            j.record_decision(&decision).unwrap();
        }
    }

    // Verify via API
    let resp = client()
        .get(format!("{}/api/journal/decisions", base_url))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "success");

    let decisions = body["decisions"].as_array().unwrap();
    assert_eq!(decisions.len(), 3, "Should have 3 decisions");

    // Verify decision content (most recent first)
    assert_eq!(decisions[0]["symbol"], "SYMBOL-2");
    assert_eq!(decisions[0]["action_taken"], "BUY");
    assert_eq!(decisions[0]["overall_conviction"], 0.7);

    assert_eq!(decisions[1]["symbol"], "SYMBOL-1");
    assert_eq!(decisions[1]["action_taken"], "SKIP");

    assert_eq!(decisions[2]["symbol"], "SYMBOL-0");
    assert_eq!(decisions[2]["action_taken"], "BUY");
    assert_eq!(decisions[2]["overall_conviction"], 0.5);

    // Verify direct DB access matches
    let j = journal.lock().await;
    let db_decisions = j.get_recent_decisions(10).unwrap();
    assert_eq!(db_decisions.len(), 3);
}
