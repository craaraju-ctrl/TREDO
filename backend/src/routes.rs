use axum::{
    routing::{get, post},
    Router,
    Json,
    extract::{State, WebSocketUpgrade, ws::{Message, WebSocket}},
    response::IntoResponse,
    http::Method,
};
use tower_http::cors::{CorsLayer, Any};
use tokio::sync::mpsc;
use serde_json::json;
use futures::{stream::StreamExt, sink::SinkExt};
use arkm_types::{ExecutionCommand, ManualOverrideRequest};
use arkm_skills::{MarketAnalysisContext, Candle};

use std::sync::Arc;
use arkm_intelligence::IntelligencePool;
use arkm_tantra::TantraService;
use arkm_core::PluginRegistry;
use arkm_data::{YahooFinanceProvider, MarketDataProvider, TimeFrame};
use arkm_journal::TradeJournal;
use arkm_autotrader::{AutoTradingLoop, AutoTradingConfig};
use arkm_learning::LearningEngine;
use arkm_stream::{StreamRegistry, StreamMessage};
use arkm_bridge::{
    RedisBridge, AgentRegistry,
    TieredCache, HierarchicalRAG, SharedMemory,
};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    pub execution_tx: mpsc::Sender<ExecutionCommand>,
    pub intelligence: Arc<IntelligencePool>,
    pub tantra: Arc<TantraService>,
    pub data_provider: Arc<YahooFinanceProvider>,
    pub journal: Arc<Mutex<TradeJournal>>,
    pub auto_trader: Arc<AutoTradingLoop>,
    pub learning_engine: Arc<Mutex<LearningEngine>>,
    pub stream_registry: Arc<StreamRegistry>,
    // Redis Bridge (Python↔Rust)
    pub redis_bridge: Arc<RedisBridge>,
    pub agent_registry: Arc<AgentRegistry>,
    pub tiered_cache: Arc<TieredCache>,
    pub rag_db: Arc<HierarchicalRAG>,
    pub shared_memory: Arc<SharedMemory>,
    // Plugin Registry (DI container for providers)
    pub registry: Arc<dyn PluginRegistry>,
}

pub fn router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers(Any);

    Router::new()
        .layer(cors)
        .route("/api/health", get(health_check))
        .route("/api/state", get(get_state))
        .route("/api/override", post(manual_override))
        .route("/api/chat", post(chat_handler))
        .route("/api/webhook/google-trading", post(google_trading_webhook))
        // Tantra
        .route("/api/tantra/status", get(tantra_status))
        .route("/api/tantra/calendar", get(tantra_calendar))
        .route("/api/tantra/dnd", post(tantra_set_dnd))
        .route("/api/tantra/tasks", get(tantra_get_tasks).post(tantra_resolve_task))
        // Skills
        .route("/api/skills/list", get(skills_list))
        .route("/api/skills/agents", get(skills_agents))
        .route("/api/skills/analyze", post(skills_analyze))
        // Market Data
        .route("/api/market/candles", post(market_candles))
        .route("/api/market/price", post(market_price))
        // Trade Journal
        .route("/api/journal/stats", get(journal_stats))
        .route("/api/journal/trades", post(journal_trades))
        .route("/api/journal/decisions", get(journal_decisions))
        .route("/api/journal/strategy-win-rates", get(journal_strategy_win_rates))
        .route("/api/journal/regime-win-rates", get(journal_regime_win_rates))
        // Auto-Trading
        .route("/api/autotrade/status", get(autotrade_status))
        .route("/api/autotrade/start", post(autotrade_start))
        .route("/api/autotrade/stop", post(autotrade_stop))
        .route("/api/autotrade/config", get(autotrade_get_config).post(autotrade_update_config))
        // Learning Engine
        .route("/api/learning/skills", get(learning_skills))
        .route("/api/learning/top-skills", get(learning_top_skills))
        .route("/api/learning/regime-optima", get(learning_regime_optima))
        .route("/api/learning/cache-stats", get(learning_cache_stats))
        .route("/api/learning/reset", post(learning_reset))
        // Bridge (Python↔Rust)
        .route("/api/bridge/status", get(bridge_status))
        .route("/api/bridge/stats", get(bridge_stats))
        .route("/api/bridge/cache/stats", get(bridge_cache_stats))
        .route("/api/bridge/agents", get(bridge_agents))
        .route("/api/bridge/memory/stats", get(bridge_memory_stats))
        .route("/api/bridge/rag/stats", get(bridge_rag_stats))
        .route("/api/bridge/rag/search", post(bridge_rag_search))
        .route("/api/bridge/store", post(bridge_store))
        .route("/api/bridge/read", post(bridge_read))
        // Provider Management (Plugin Architecture)
        .route("/api/providers/list", get(providers_list))
        .route("/api/providers/agents", get(providers_agents))
        .route("/api/providers/llms", get(providers_llms))
        .route("/api/providers/swap-agent", post(providers_swap_agent))
        .route("/api/providers/swap-llm", post(providers_swap_llm))
        // WebSocket
        .route("/ws", get(ws_handler))
        .with_state(state)
}

// ── Health & State ─────────────────────────────────────────────────────────

async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "operational",
        "engine": "ARKM Axum Core v2",
        "features": {
            "websocket_streaming": true,
            "self_learning": true,
            "sub_agent_framework": true,
            "kv_cache": true
        }
    }))
}

async fn get_state(State(state): State<AppState>) -> impl IntoResponse {
    let stream_stats = state.stream_registry.stats();
    let cache_stats = state.intelligence.cache_stats();
    let learning_trades = {
        let engine = state.learning_engine.lock().await;
        engine.total_trades()
    };

    Json(json!({
        "status": "active",
        "version": "v2",
        "engines": [
            "ExecutionEngine",
            "IntelligencePool",
            "ExchangeAdapters",
            "AutoTradingLoop",
            "SelfLearningEngine",
            "SubAgentFramework",
            "WebSocketStreaming"
        ],
        "stream_stats": stream_stats,
        "kv_cache_stats": cache_stats,
        "learning_trades": learning_trades,
        "note": "Connect to /ws for live streaming"
    }))
}

// ── Chat ───────────────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct ChatRequest {
    prompt: String,
}

async fn chat_handler(
    State(state): State<AppState>,
    Json(payload): Json<ChatRequest>,
) -> impl IntoResponse {
    match state.intelligence.query_analyst(&payload.prompt).await {
        Ok(reply) => Json(json!({ "status": "success", "reply": reply })),
        Err(e) => Json(json!({ "status": "error", "message": e })),
    }
}

// ── Tantra ─────────────────────────────────────────────────────────────────

async fn tantra_status(State(state): State<AppState>) -> impl IntoResponse {
    let dnd = state.tantra.is_dnd_active();
    let tasks_count = state.tantra.tasks.len();
    Json(json!({
        "status": "success",
        "dnd_active": dnd,
        "active_tasks_count": tasks_count,
        "safety_index": if dnd { "HIGH_GUARD" } else { "STANDARD" }
    }))
}

async fn tantra_calendar(State(state): State<AppState>) -> impl IntoResponse {
    let mut events = Vec::new();
    for entry in state.tantra.events.iter() {
        events.push(entry.value().clone());
    }
    Json(json!({ "status": "success", "events": events }))
}

#[derive(serde::Deserialize)]
struct DndRequest {
    active: bool,
}

async fn tantra_set_dnd(
    State(state): State<AppState>,
    Json(payload): Json<DndRequest>,
) -> impl IntoResponse {
    state.tantra.set_dnd(payload.active);
    Json(json!({ "status": "success", "dnd_active": payload.active }))
}

async fn tantra_get_tasks(State(state): State<AppState>) -> impl IntoResponse {
    let mut tasks = Vec::new();
    for entry in state.tantra.tasks.iter() {
        tasks.push(entry.value().clone());
    }
    Json(json!({ "status": "success", "tasks": tasks }))
}

#[derive(serde::Deserialize)]
struct ResolveTaskRequest {
    id: String,
}

async fn tantra_resolve_task(
    State(state): State<AppState>,
    Json(payload): Json<ResolveTaskRequest>,
) -> impl IntoResponse {
    let resolved = state.tantra.resolve_task(&payload.id);
    if resolved {
        Json(json!({ "status": "success", "message": "Task resolved successfully" }))
    } else {
        Json(json!({ "status": "error", "message": "Task not found" }))
    }
}

// ── Manual Override ────────────────────────────────────────────────────────

async fn manual_override(
    State(state): State<AppState>,
    Json(payload): Json<ManualOverrideRequest>,
) -> impl IntoResponse {
    println!(
        "[ManualOverride] Overriding trade for symbol: {}, side: {}, amount: {}",
        payload.symbol, payload.side, payload.amount
    );
    let _ = state
        .execution_tx
        .send(ExecutionCommand::UpdateBalance("USDT".to_string(), 10000.0))
        .await;
    Json(json!({ "status": "success", "message": "Manual override processed" }))
}

// ── Google cTrading Webhook ────────────────────────────────────────────────

#[derive(serde::Deserialize, Clone)]
struct GoogleTradingRequest {
    symbol: String,
    action: String,   // "BUY" or "SELL"
    amount: f64,
    price: Option<f64>,
    bypass_safety: Option<bool>,
}

#[derive(serde::Deserialize)]
struct GeminiConfirmationResponse {
    status: String,       // "Approved" or "Rejected"
    conviction: f64,      // 0.0 to 1.0
    reasoning: String,
}

async fn google_trading_webhook(
    State(state): State<AppState>,
    Json(payload): Json<GoogleTradingRequest>,
) -> impl IntoResponse {
    println!(
        "[GoogleTradingWebhook] Received call from Google API: symbol={}, action={}, amount={}",
        payload.symbol, payload.action, payload.amount
    );

    // 1. Fetch historical candles for Technical/Chart analysis
    let timeframe = TimeFrame::Hour1;
    let candles = match state.data_provider.fetch_candles(&payload.symbol, timeframe).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[GoogleTradingWebhook] Failed to fetch candles: {}", e);
            vec![]
        }
    };

    let current_price = payload.price.or_else(|| candles.last().map(|c| c.close)).unwrap_or(100.0);

    // 2. Technical Analysis / Skills check
    let context = MarketAnalysisContext {
        symbol: payload.symbol.clone(),
        candles,
        current_price,
        cash_available: 100000.0,
        portfolio_value: 100000.0,
        exposure: 0.0,
        open_positions: std::collections::HashMap::new(),
    };
    let technical_analysis = state.intelligence.analyze_with_skills(context).await;

    // 3. Fundamental News check
    let mut active_news = Vec::new();
    for entry in state.tantra.news.iter() {
        active_news.push(entry.value().headline.clone());
    }

    // 4. Calendar check (Bypass if requested)
    let bypass = payload.bypass_safety.unwrap_or(false);
    let mut active_events = Vec::new();
    let dnd_active = if bypass {
        false
    } else {
        for entry in state.tantra.events.iter() {
            active_events.push(format!("{} (DND: {})", entry.value().title, entry.value().is_dnd));
        }
        state.tantra.is_dnd_active()
    };

    // 5. Build prompt explaining scenarios to Gemini for confirmation
    let prompt = format!(
        "You are Hermes, the autonomous trading coordinator. Review this incoming cTrading call:\n\n\
         INCOMING TRADE CALL:\n\
         - Symbol: {}\n\
         - Side/Action: {}\n\
         - Size/Amount: {}\n\
         - Price: {}\n\n\
         HERMES GATHERED CROSSCHECKS:\n\
         1. CHART & TECHNICALS:\n\
         - Sentiment: {:?}\n\
         - Technical Conviction: {:.0}%\n\
         - Signals Fired: {} bullish, {} bearish, {} neutral\n\n\
         2. NEWS & MARKET INTEL:\n\
         - Headlines: {:?}\n\n\
         3. CALENDAR LOCKS & SAFETY CONTROLLER:\n\
         - Scheduled Events: {:?}\n\
         - Active Safety DND Mode: {}\n\n\
         DECISION POLICY:\n\
         If DND Mode is active, or if calendar event status indicates risk, or technical conviction is very low (< 0.2), prioritize REJECTING the trade. Otherwise, check if technical indicator direction matches the trade call action (BUY/SELL).\n\n\
         You MUST respond in strict JSON format: \n\
         {{ \"status\": \"Approved\" | \"Rejected\", \"conviction\": 0.0 to 1.0, \"reasoning\": \"Your concise explanation\" }}",
        payload.symbol, payload.action, payload.amount, current_price,
        technical_analysis.overall_direction, technical_analysis.overall_conviction * 100.0,
        technical_analysis.bullish_signals, technical_analysis.bearish_signals, technical_analysis.neutral_signals,
        active_news, active_events, dnd_active
    );

    // 6. Explain scenarios to Gemini LLM for confirmation
    let (status, conviction, reasoning) = match state.intelligence.query_analyst(&prompt).await {
        Ok(reply) => {
            println!("[GoogleTradingWebhook] Gemini confirmation reply: {}", reply);
            if let Ok(parsed) = serde_json::from_str::<GeminiConfirmationResponse>(&reply) {
                (parsed.status, parsed.conviction, parsed.reasoning)
            } else {
                if reply.contains("Approved") || reply.contains("APPROVED") {
                    ("Approved".to_string(), 0.75, "Auto-approved via confirmation fallback parsing".to_string())
                } else {
                    ("Rejected".to_string(), 0.1, "Rejected due to confirmation fallback failure".to_string())
                }
            }
        }
        Err(e) => {
            eprintln!("[GoogleTradingWebhook] Gemini query failed: {}", e);
            ("Rejected".to_string(), 0.0, "Gemini confirmation API call timed out".to_string())
        }
    };

    let approved = status == "Approved" || status == "APPROVED";

    // 7. Persistent logging in SQLite Trade Journal
    let action_side = if payload.action.to_uppercase() == "BUY" { "BUY" } else { "SELL" };
    let decision_rec = arkm_journal::DecisionRecord {
        id: uuid::Uuid::new_v4().to_string(),
        symbol: payload.symbol.clone(),
        timestamp: chrono::Utc::now(),
        overall_conviction: conviction,
        overall_direction: if approved { action_side.to_string() } else { "REJECTED".to_string() },
        market_regime: "trending_bullish".to_string(),
        action_taken: if approved { action_side.to_string() } else { "SKIP".to_string() },
        reason: reasoning.clone(),
        bullish_signals: technical_analysis.bullish_signals,
        bearish_signals: technical_analysis.bearish_signals,
        neutral_signals: technical_analysis.neutral_signals,
    };
    let _ = state.journal.lock().await.record_decision(&decision_rec);

    if approved {
        let decision = arkm_types::TradeDecision {
            id: uuid::Uuid::new_v4(),
            symbol: payload.symbol.clone(),
            action: action_side.to_string(),
            amount: payload.amount,
            price: current_price,
            conviction,
            reasoning: format!("[Google cTrading Call Verified] {}", reasoning),
            status: arkm_types::DecisionStatus::Approved,
            timestamp: chrono::Utc::now(),
        };

        let _ = state.execution_tx.send(ExecutionCommand::Execute(decision)).await;

        state.stream_registry.global().alert(
            "success",
            &format!("🟢 Auto-Traded {} {} via Google API + Gemini Confirmation!", payload.action.to_uppercase(), payload.symbol)
        );

        Json(json!({
            "status": "Approved",
            "message": "Google cTrading call verified by Hermes and automatically executed.",
            "conviction": conviction,
            "reasoning": reasoning,
            "trade": {
                "symbol": payload.symbol,
                "action": action_side,
                "amount": payload.amount,
                "price": current_price
            }
        }))
    } else {
        state.stream_registry.global().alert(
            "warning",
            &format!("🔴 Rejected Google cTrading call for {} due to Gemini Safety Lock", payload.symbol)
        );

        Json(json!({
            "status": "Rejected",
            "message": "Google cTrading call was rejected by Hermes / Gemini safety review.",
            "conviction": conviction,
            "reasoning": reasoning
        }))
    }
}

// ── Skills ─────────────────────────────────────────────────────────────────

async fn skills_list(State(state): State<AppState>) -> impl IntoResponse {
    let skills = state.intelligence.list_skills().await;
    Json(json!({
        "status": "success",
        "total_skills": skills.len(),
        "skills": skills
    }))
}

async fn skills_agents(State(state): State<AppState>) -> impl IntoResponse {
    let agents = state.intelligence.agent_info().await;
    Json(json!({
        "status": "success",
        "total_agents": agents.len(),
        "agents": agents
    }))
}

#[derive(serde::Deserialize)]
struct SkillsAnalyzeRequest {
    symbol: String,
    current_price: f64,
    cash_available: Option<f64>,
    portfolio_value: Option<f64>,
    candles: Option<Vec<CandlePayload>>,
}

#[derive(serde::Deserialize, Clone)]
struct CandlePayload {
    time: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

async fn skills_analyze(
    State(state): State<AppState>,
    Json(payload): Json<SkillsAnalyzeRequest>,
) -> impl IntoResponse {
    let candles: Vec<Candle> = payload
        .candles
        .unwrap_or_default()
        .into_iter()
        .map(|c| Candle {
            time: c.time,
            open: c.open,
            high: c.high,
            low: c.low,
            close: c.close,
            volume: c.volume,
        })
        .collect();

    let context = MarketAnalysisContext {
        symbol: payload.symbol,
        candles,
        current_price: payload.current_price,
        cash_available: payload.cash_available.unwrap_or(100000.0),
        portfolio_value: payload.portfolio_value.unwrap_or(100000.0),
        exposure: 0.0,
        open_positions: std::collections::HashMap::new(),
    };

    // Run both flat analysis and orchestrated sub-agent analysis
    let analysis = state.intelligence.analyze_with_skills(context.clone()).await;
    let orchestrated = state.intelligence.analyze_orchestrated(&context).await;

    Json(json!({
        "status": "success",
        "analysis": analysis,
        "orchestrated": orchestrated
    }))
}

// ── Market Data ────────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct MarketCandlesRequest {
    symbol: String,
    timeframe: Option<String>, // "1m", "5m", "15m", "1h", "1d"
}

async fn market_candles(
    State(state): State<AppState>,
    Json(payload): Json<MarketCandlesRequest>,
) -> impl IntoResponse {
    let timeframe = match payload.timeframe.as_deref() {
        Some("1m") => TimeFrame::Min1,
        Some("5m") => TimeFrame::Min5,
        Some("15m") => TimeFrame::Min15,
        Some("30m") => TimeFrame::Min30,
        Some("4h") => TimeFrame::Hour4,
        Some("1d") => TimeFrame::Day1,
        _ => TimeFrame::Hour1,
    };

    match state.data_provider.fetch_candles(&payload.symbol, timeframe).await {
        Ok(candles) => {
            // Broadcast to WebSocket
            state.stream_registry.broadcast(
                Some(&payload.symbol),
                StreamMessage::Alert {
                    severity: "info".to_string(),
                    message: format!("Fetched {} {} candles for {}", candles.len(), timeframe.label(), payload.symbol),
                    timestamp: chrono::Utc::now(),
                },
            );

            Json(json!({
                "status": "success",
                "symbol": payload.symbol,
                "timeframe": timeframe.label(),
                "candles": candles,
                "count": candles.len()
            }))
        }
        Err(e) => Json(json!({
            "status": "error",
            "message": e
        })),
    }
}

#[derive(serde::Deserialize)]
struct MarketPriceRequest {
    symbol: String,
}

async fn market_price(
    State(state): State<AppState>,
    Json(payload): Json<MarketPriceRequest>,
) -> impl IntoResponse {
    match state.data_provider.fetch_current_price(&payload.symbol).await {
        Ok(price) => {
            // Broadcast price tick to WebSocket
            state.stream_registry.broadcast(
                Some(&payload.symbol),
                StreamMessage::PriceTick {
                    symbol: payload.symbol.clone(),
                    price,
                    change_24h: 0.0,
                    volume_24h: 0.0,
                    timestamp: chrono::Utc::now(),
                },
            );

            Json(json!({
                "status": "success",
                "symbol": payload.symbol,
                "price": price,
            }))
        }
        Err(e) => Json(json!({
            "status": "error",
            "message": e
        })),
    }
}

// ── Trade Journal ──────────────────────────────────────────────────────────

async fn journal_stats(State(state): State<AppState>) -> impl IntoResponse {
    let journal = state.journal.lock().await;
    match journal.get_performance_stats() {
        Ok(stats) => Json(json!({ "status": "success", "stats": stats })),
        Err(e) => Json(json!({ "status": "error", "message": e.to_string() })),
    }
}

#[derive(serde::Deserialize)]
struct JournalTradesQuery {
    limit: Option<u64>,
    offset: Option<u64>,
}

async fn journal_trades(
    State(state): State<AppState>,
    Json(payload): Json<JournalTradesQuery>,
) -> impl IntoResponse {
    let journal = state.journal.lock().await;
    let limit = payload.limit.unwrap_or(50);
    let offset = payload.offset.unwrap_or(0);
    match journal.get_trade_history(limit, offset) {
        Ok(trades) => Json(json!({ "status": "success", "trades": trades })),
        Err(e) => Json(json!({ "status": "error", "message": e.to_string() })),
    }
}

async fn journal_decisions(State(state): State<AppState>) -> impl IntoResponse {
    let journal = state.journal.lock().await;
    match journal.get_recent_decisions(50) {
        Ok(decisions) => Json(json!({ "status": "success", "decisions": decisions })),
        Err(e) => Json(json!({ "status": "error", "message": e.to_string() })),
    }
}

async fn journal_strategy_win_rates(State(state): State<AppState>) -> impl IntoResponse {
    let journal = state.journal.lock().await;
    match journal.get_strategy_win_rates() {
        Ok(rates) => Json(json!({ "status": "success", "strategy_win_rates": rates })),
        Err(e) => Json(json!({ "status": "error", "message": e.to_string() })),
    }
}

async fn journal_regime_win_rates(State(state): State<AppState>) -> impl IntoResponse {
    let journal = state.journal.lock().await;
    match journal.get_regime_win_rates() {
        Ok(rates) => Json(json!({ "status": "success", "regime_win_rates": rates })),
        Err(e) => Json(json!({ "status": "error", "message": e.to_string() })),
    }
}

// ── Auto-Trading ───────────────────────────────────────────────────────────

async fn autotrade_status(State(state): State<AppState>) -> impl IntoResponse {
    let trading_state = state.auto_trader.get_state().await;
    let learning_skills = {
        let engine = state.learning_engine.lock().await;
        engine.all_skill_performance()
    };
    Json(json!({
        "status": "success",
        "trading_state": trading_state,
        "learning_skills_sample": learning_skills.iter().take(5).collect::<Vec<_>>()
    }))
}

async fn autotrade_start(State(state): State<AppState>) -> impl IntoResponse {
    state.auto_trader.set_enabled(true).await;
    state.stream_registry.global().alert("success", "Auto-trading started");
    Json(json!({ "status": "success", "message": "Auto-trading started"}))
}

async fn autotrade_stop(State(state): State<AppState>) -> impl IntoResponse {
    state.auto_trader.set_enabled(false).await;
    state.stream_registry.global().alert("warning", "Auto-trading stopped");
    Json(json!({ "status": "success", "message": "Auto-trading stopped"}))
}

async fn autotrade_get_config(State(state): State<AppState>) -> impl IntoResponse {
    let trading_state = state.auto_trader.get_state().await;
    Json(json!({
        "status": "success",
        "config": {
            "enabled": trading_state.enabled,
            "paper_trading": trading_state.paper_trading,
            "symbols": trading_state.symbols,
            "analysis_interval_secs": trading_state.analysis_interval_secs,
            "adaptive_weights_enabled": trading_state.adaptive_weights_enabled,
            "total_learned_trades": trading_state.total_learned_trades,
            "regime_optimization_enabled": trading_state.regime_optimization_enabled,
        }
    }))
}

#[derive(serde::Deserialize)]
struct AutoTradeConfigUpdate {
    enabled: Option<bool>,
    paper_trading: Option<bool>,
    symbols: Option<Vec<String>>,
    analysis_interval_secs: Option<u64>,
}

async fn autotrade_update_config(
    State(state): State<AppState>,
    Json(payload): Json<AutoTradeConfigUpdate>,
) -> impl IntoResponse {
    let current = state.auto_trader.get_state().await;
    let config = AutoTradingConfig {
        enabled: payload.enabled.unwrap_or(current.enabled),
        paper_trading: payload.paper_trading.unwrap_or(current.paper_trading),
        symbols: payload.symbols.unwrap_or(current.symbols),
        analysis_interval_secs: payload.analysis_interval_secs.unwrap_or(current.analysis_interval_secs),
        ..Default::default()
    };
    state.auto_trader.update_config(config).await;
    state.stream_registry.global().alert("info", "Auto-trading configuration updated");
    Json(json!({ "status": "success", "message": "Configuration updated" }))
}

// ── Learning Engine ────────────────────────────────────────────────────────

async fn learning_skills(State(state): State<AppState>) -> impl IntoResponse {
    let engine = state.learning_engine.lock().await;
    let skills = engine.all_skill_performance();
    Json(json!({
        "status": "success",
        "total_skills": skills.len(),
        "total_trades": engine.total_trades(),
        "skills": skills
    }))
}

async fn learning_top_skills(State(state): State<AppState>) -> impl IntoResponse {
    let engine = state.learning_engine.lock().await;
    let top = engine.top_skills(10);
    let worst = engine.worst_skills(5);
    Json(json!({
        "status": "success",
        "top_performers": top,
        "underperformers": worst
    }))
}

async fn learning_regime_optima(State(state): State<AppState>) -> impl IntoResponse {
    let engine = state.learning_engine.lock().await;
    let optima = engine.regime_optima();
    Json(json!({
        "status": "success",
        "regime_optimizations": optima
    }))
}

async fn learning_cache_stats(State(state): State<AppState>) -> impl IntoResponse {
    let kv_stats = state.intelligence.cache_stats();
    let tiered_stats = state.tiered_cache.stats().await;
    Json(json!({
        "status": "success",
        "kv_cache": kv_stats,
        "tiered_cache": tiered_stats
    }))
}

// ── Bridge (Python↔Rust) Routes ───────────────────────────────────────────

async fn bridge_status(State(state): State<AppState>) -> impl IntoResponse {
    let agents = state.agent_registry.stats().await;
    let stats = state.redis_bridge.stats().await;
    Json(json!({
        "status": "success",
        "bridge_id": state.redis_bridge.bridge_id(),
        "agents": agents,
        "bridge_stats": stats
    }))
}

async fn bridge_stats(State(state): State<AppState>) -> impl IntoResponse {
    let stats = state.redis_bridge.stats().await;
    Json(json!({
        "status": "success",
        "stats": stats
    }))
}

async fn bridge_cache_stats(State(state): State<AppState>) -> impl IntoResponse {
    let stats = state.tiered_cache.stats().await;
    Json(json!({
        "status": "success",
        "cache_stats": stats
    }))
}

async fn bridge_agents(State(state): State<AppState>) -> impl IntoResponse {
    let registry_stats = state.agent_registry.stats().await;
    Json(json!({
        "status": "success",
        "registry_stats": registry_stats
    }))
}

async fn bridge_memory_stats(State(state): State<AppState>) -> impl IntoResponse {
    let stats = state.shared_memory.stats().await;
    Json(json!({
        "status": "success",
        "memory_stats": stats
    }))
}

async fn bridge_rag_stats(State(state): State<AppState>) -> impl IntoResponse {
    let stats = state.rag_db.stats().await;
    Json(json!({
        "status": "success",
        "rag_stats": stats
    }))
}

#[derive(serde::Deserialize)]
struct RAGSearchRequest {
    query: String,
    agent_id: Option<String>,
    limit: Option<usize>,
}

async fn bridge_rag_search(
    State(state): State<AppState>,
    Json(payload): Json<RAGSearchRequest>,
) -> impl IntoResponse {
    match state.rag_db.search(
        &payload.query,
        payload.agent_id.as_deref(),
        payload.limit.unwrap_or(10),
    ).await {
        Ok(results) => Json(json!({
            "status": "success",
            "results": results,
            "count": results.len()
        })),
        Err(e) => Json(json!({
            "status": "error",
            "message": e
        })),
    }
}

#[derive(serde::Deserialize)]
struct BridgeStoreRequest {
    namespace: String,
    key: String,
    value: serde_json::Value,
}

async fn bridge_store(
    State(state): State<AppState>,
    Json(payload): Json<BridgeStoreRequest>,
) -> impl IntoResponse {
    match state.shared_memory.write(
        &payload.namespace,
        &payload.key,
        payload.value,
    ).await {
        Ok(block) => Json(json!({
            "status": "success",
            "block_id": block.id,
            "size_bytes": block.size_bytes
        })),
        Err(e) => Json(json!({
            "status": "error",
            "message": e
        })),
    }
}

#[derive(serde::Deserialize)]
struct BridgeReadRequest {
    namespace: String,
    key: String,
}

async fn bridge_read(
    State(state): State<AppState>,
    Json(payload): Json<BridgeReadRequest>,
) -> impl IntoResponse {
    match state.shared_memory.read(
        &payload.namespace,
        &payload.key,
    ).await {
        Ok(Some(block)) => Json(json!({
            "status": "success",
            "block": block
        })),
        Ok(None) => Json(json!({
            "status": "not_found",
            "message": "Key not found in shared memory"
        })),
        Err(e) => Json(json!({
            "status": "error",
            "message": e
        })),
    }
}

// ── Provider Management (Plugin Architecture) Routes ───────────────────────

/// List all registered providers
async fn providers_list(State(state): State<AppState>) -> impl IntoResponse {
    let agents = state.registry.list_agents();
    let llms = state.registry.list_llms();
    let agent_provider = state.intelligence.agent_provider();
    let llm_provider = state.intelligence.llm_provider();
    Json(json!({
        "status": "success",
        "architecture": "plugin",
        "agents": agents,
        "llms": llms,
        "current_agent": agent_provider.provider_name(),
        "current_llm": llm_provider.provider_name(),
        "note": "Use /api/providers/swap-agent or /api/providers/swap-llm to swap at runtime"
    }))
}

/// List all registered agent providers with details
async fn providers_agents(State(state): State<AppState>) -> impl IntoResponse {
    let agents = state.registry.list_agents();
    let agent_provider = state.intelligence.agent_provider();
    let current = agent_provider.provider_name().to_string();
    Json(json!({
        "status": "success",
        "available_agents": agents,
        "current_active": current,
        "total": agents.len()
    }))
}

/// List all registered LLM providers with details
async fn providers_llms(State(state): State<AppState>) -> impl IntoResponse {
    let llms = state.registry.list_llms();
    let llm_provider = state.intelligence.llm_provider();
    let current = llm_provider.provider_name().to_string();
    Json(json!({
        "status": "success",
        "available_llms": llms,
        "current_active": current,
        "total": llms.len()
    }))
}

#[derive(serde::Deserialize)]
struct SwapAgentRequest {
    name: String,
}

/// Swap the active agent provider at runtime (no restart needed)
/// Uses the IntelligencePool's RwLock-based hot-swap mechanism.
async fn providers_swap_agent(
    State(state): State<AppState>,
    Json(payload): Json<SwapAgentRequest>,
) -> impl IntoResponse {
    if let Some(agent) = state.registry.get_agent(&payload.name) {
        let name = agent.provider_name().to_string();
        state.intelligence.swap_agent(agent);
        println!("[ProviderManager] 🔄 Agent provider hot-swapped to '{}'", payload.name);
        Json(json!({
            "status": "success",
            "message": format!("Agent provider hot-swapped to '{}'", payload.name),
            "current_active": name,
            "available_agents": state.registry.list_agents(),
        }))
    } else {
        Json(json!({
            "status": "error",
            "message": format!("Agent '{}' not found in registry", payload.name),
            "available_agents": state.registry.list_agents()
        }))
    }
}

#[derive(serde::Deserialize)]
struct SwapLLMRequest {
    name: String,
}

/// Swap the active LLM provider at runtime (no restart needed)
/// Uses the IntelligencePool's RwLock-based hot-swap mechanism.
async fn providers_swap_llm(
    State(state): State<AppState>,
    Json(payload): Json<SwapLLMRequest>,
) -> impl IntoResponse {
    if let Some(llm) = state.registry.get_llm(&payload.name) {
        let name = llm.provider_name().to_string();
        state.intelligence.swap_llm(llm);
        println!("[ProviderManager] 🔄 LLM provider hot-swapped to '{}'", payload.name);
        Json(json!({
            "status": "success",
            "message": format!("LLM provider hot-swapped to '{}'", payload.name),
            "current_active": name,
            "available_llms": state.registry.list_llms(),
        }))
    } else {
        Json(json!({
            "status": "error",
            "message": format!("LLM '{}' not found in registry", payload.name),
            "available_llms": state.registry.list_llms()
        }))
    }
}

async fn learning_reset(State(state): State<AppState>) -> impl IntoResponse {
    let mut engine = state.learning_engine.lock().await;
    engine.reset();
    state.stream_registry.global().alert("info", "Learning engine reset");
    Json(json!({ "status": "success", "message": "Learning engine reset successfully" }))
}

// ── WebSocket ──────────────────────────────────────────────────────────────

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle a single WebSocket connection — subscribes to the broadcast hub
/// and forwards all stream messages to the client as JSON.
async fn handle_socket(mut socket: WebSocket, state: AppState) {
    println!("[WebSocket] Client connected, subscribing to broadcast hub");
    state.stream_registry.global().alert("info", "WebSocket client connected");

    let mut rx = state.stream_registry.global().subscribe();

    // Send initial connection confirmation
    let welcome = serde_json::to_string(&StreamMessage::Alert {
        severity: "success".to_string(),
        message: "Connected to ARKM v2 streaming hub. Sub-agents: TechnicalAnalyst, RiskManager, PortfolioManager, MarketDataAgent".to_string(),
        timestamp: chrono::Utc::now(),
    }).unwrap_or_default();

    if socket.send(Message::Text(welcome)).await.is_err() {
        println!("[WebSocket] Client disconnected during welcome");
        return;
    }

    // Use a channel to send messages to the broadcast task (avoids use-after-move)
    let (ws_tx, mut ws_rx) = tokio::sync::mpsc::channel::<Message>(128);
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Broadcast task: forward messages from both the hub and the mpsc channel to the WebSocket
    let broadcast_task = tokio::spawn(async move {
        use tokio::select;
        loop {
            select! {
                // From the broadcast hub
                msg = rx.recv() => {
                    match msg {
                        Ok(msg) => if let Ok(json_str) = serde_json::to_string(&msg) {
                            if ws_sender.send(Message::Text(json_str)).await.is_err() {
                                break;
                            }
                        },
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    }
                }
                // From the mpsc channel (pongs, acks)
                Some(msg) = ws_rx.recv() => {
                    if ws_sender.send(msg).await.is_err() {
                        break;
                    }
                }
                else => break,
            }
        }
    });

    // Read task: handle incoming messages (pings, commands from client)
    while let Some(Ok(msg)) = ws_receiver.next().await {
        match msg {
            Message::Ping(data) => {
                let result = ws_tx.send(Message::Pong(data)).await;
                if result.is_err() {
                    break;
                }
            }
            Message::Text(text) => {
                // Client sent a text message — just log it
                println!("[WebSocket] Received from client: {}", text);

                // Acknowledge via the channel
                let ack = json!({
                    "type": "ack",
                    "data": { "received": true, "message": text }
                });
                if ws_tx.send(Message::Text(serde_json::to_string(&ack).expect("ACK JSON serialization should not fail"))).await.is_err() {
                    break;
                }
            }
            Message::Close(_) => {
                println!("[WebSocket] Client sent close frame");
                break;
            }
            _ => {}
        }
    }

    // Cleanup
    broadcast_task.abort();
    println!("[WebSocket] Client disconnected");
    state.stream_registry.global().alert("info", "WebSocket client disconnected");
}
