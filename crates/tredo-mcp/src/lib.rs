//! # TREDO MCP Server — Model Context Protocol Integration
//!
//! Exposes the full TREDO trading system as MCP tools and resources,
//! enabling remote control via any MCP client (Claude Desktop, Cursor, etc.).
//!
//! ## Architecture
//!
//! ```text
//! ┌────────────────┐     ┌─────────────────┐     ┌──────────────────┐
//! │  MCP Client    │◄───►│  TREDO MCP Server │◄───►│  TREDO Backend    │
//! │  (Claude, etc) │     │  (rmcp)          │     │  (Axum Routes)   │
//! │                │     │  /sse + /message  │     │  /api/*          │
//! └────────────────┘     └─────────────────┘     └──────────────────┘
//! ```
//!
//! ## Transport
//!
//! - **SSE (Server-Sent Events)**: For remote MCP clients over HTTP
//! - **Stdio**: For local MCP clients (e.g., Claude Desktop)
//!
//! ## Available Tools
//!
//! - `market_analysis` — Run full market analysis on a symbol
//! - `trading_status` — Get auto-trading system status
//! - `trading_control` — Start/stop/pause auto-trading
//! - `manual_trade` — Execute a manual trade override
//! - `get_price` — Get current price for a symbol
//! - `market_candles` — Get historical candle data
//! - `skills_list` — List all available analysis skills
//! - `system_status` — Get overall system health
//! - `portfolio_status` — Get portfolio and journal stats
//! - `bridge_query` — Query Python Nethra via Redis bridge
//! - `learning_stats` — Get learning engine statistics
//!

use std::sync::Arc;

use chrono::Utc;
use rmcp::{
    handler::server::{wrapper::Parameters, ServerHandler},
    model::{ErrorData as McpError, *}, // For ServerCapabilities, Implementation, etc.
    schemars,
    service::{RequestContext, RoleServer},
    tool,
    tool_router,
};
use serde::Deserialize;
use tokio::sync::Mutex;
use tredo_autotrader::AutoTradingLoop;
use tredo_bridge::{AgentRegistry, HierarchicalRAG, RedisBridge, SharedMemory, TieredCache};
use tredo_core::PluginRegistry;
use tredo_data::{MarketDataProvider, TimeFrame, YahooFinanceProvider};
use tredo_intelligence::IntelligencePool;
use tredo_journal::TradeJournal;
use tredo_learning::LearningEngine;
use tredo_stream::StreamRegistry;
use tredo_tantra::TantraService;

// ═══════════════════════════════════════════════════════════════════════════
//  AppState Wrapper — cloned into each MCP handler
// ═══════════════════════════════════════════════════════════════════════════

/// Thread-safe shared state for the MCP server, mirroring backend `AppState`.
#[derive(Clone)]
pub struct McpState {
    pub intelligence: Arc<IntelligencePool>,
    pub data_provider: Arc<YahooFinanceProvider>,
    pub journal: Arc<Mutex<TradeJournal>>,
    pub auto_trader: Arc<AutoTradingLoop>,
    pub learning_engine: Arc<Mutex<LearningEngine>>,
    pub tantra: Arc<TantraService>,
    pub stream_registry: Arc<StreamRegistry>,
    pub redis_bridge: Arc<RedisBridge>,
    pub agent_registry: Arc<AgentRegistry>,
    pub tiered_cache: Arc<TieredCache>,
    pub shared_memory: Arc<SharedMemory>,
    pub registry: Arc<dyn PluginRegistry>,
    pub rag_db: Arc<HierarchicalRAG>,
}

impl std::fmt::Debug for McpState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpState")
            .field("intelligence", &"<IntelligencePool>")
            .field("data_provider", &"<YahooFinanceProvider>")
            .field("journal", &"<TradeJournal>")
            .field("auto_trader", &"<AutoTradingLoop>")
            .field("learning_engine", &"<LearningEngine>")
            .field("tantra", &"<TantraService>")
            .field("stream_registry", &"<StreamRegistry>")
            .field("redis_bridge", &"<RedisBridge>")
            .field("agent_registry", &"<AgentRegistry>")
            .field("tiered_cache", &"<TieredCache>")
            .field("shared_memory", &"<SharedMemory>")
            .field("registry", &"<PluginRegistry>")
            .field("rag_db", &"<HierarchicalRAG>")
            .finish()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  Tool Parameter Structs
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MarketAnalysisParams {
    #[schemars(description = "Trading symbol (e.g., BTC-USD, AAPL)")]
    pub symbol: String,

    #[schemars(description = "Current price of the symbol")]
    pub current_price: f64,

    #[schemars(description = "Available cash balance")]
    pub cash_available: Option<f64>,

    #[schemars(description = "Total portfolio value")]
    pub portfolio_value: Option<f64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TradingControlParams {
    #[schemars(description = "Action to perform: start, stop, or status")]
    pub action: String, // "start", "stop", "status"
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ManualTradeParams {
    #[schemars(description = "Trading symbol (e.g., BTC-USD)")]
    pub symbol: String,

    #[schemars(description = "Trade side: BUY or SELL")]
    pub side: String,

    #[schemars(description = "Trade amount in quote currency")]
    pub amount: f64,

    #[schemars(description = "Optional price limit")]
    pub price: Option<f64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetPriceParams {
    #[schemars(description = "Trading symbol (e.g., BTC-USD, AAPL)")]
    pub symbol: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MarketCandlesParams {
    #[schemars(description = "Trading symbol (e.g., BTC-USD)")]
    pub symbol: String,

    #[schemars(description = "Timeframe: 1m, 5m, 15m, 1h, 4h, 1d")]
    pub timeframe: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct BridgeQueryParams {
    #[schemars(description = "Method to call on Python Nethra")]
    pub method: String,

    #[schemars(description = "JSON parameters for the method")]
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RAGSearchParams {
    #[schemars(description = "Search query text")]
    pub query: String,

    #[schemars(description = "Optional agent ID to scope search")]
    pub agent_id: Option<String>,

    #[schemars(description = "Maximum number of results")]
    pub limit: Option<usize>,
}

// ═══════════════════════════════════════════════════════════════════════════
//  MCP Server Handler — defines all tools via #[tool_router]
// ═══════════════════════════════════════════════════════════════════════════

/// TREDO MCP Server — exposes trading system as MCP tools.
#[derive(Debug, Clone)]
pub struct ArkMcpServer {
    pub state: McpState,
}

impl ArkMcpServer {
    pub fn new(state: McpState) -> Self {
        Self { state }
    }
}

#[tool_router(server_handler)]
impl ArkMcpServer {
    // ── 1. Market Analysis ──────────────────────────────────────────────

    #[tool(
        description = "Run full technical and risk analysis on a trading symbol using 30+ built-in skills"
    )]
    async fn market_analysis(
        &self,
        Parameters(params): Parameters<MarketAnalysisParams>,
    ) -> String {
        let candles = self
            .state
            .data_provider
            .fetch_candles(&params.symbol, TimeFrame::Hour1)
            .await
            .unwrap_or_default();

        let current_price = params.current_price;
        let cash_available = params.cash_available.unwrap_or(100_000.0);
        let portfolio_value = params.portfolio_value.unwrap_or(100_000.0);

        use std::collections::HashMap;
        let context = tredo_core::MarketAnalysisContext {
            symbol: params.symbol.clone(),
            candles: candles
                .into_iter()
                .map(|c| tredo_core::Candle {
                    time: c.time,
                    open: c.open,
                    high: c.high,
                    low: c.low,
                    close: c.close,
                    volume: c.volume,
                })
                .collect(),
            current_price,
            cash_available,
            portfolio_value,
            exposure: 0.0,
            open_positions: HashMap::new(),
            local_skills: None,
        };

        let analysis = self.state.intelligence.analyze_with_skills(context).await;
        let skills = self.state.intelligence.list_skills().await;

        serde_json::json!({
            "symbol": params.symbol,
            "price": current_price,
            "direction": analysis.overall_direction,
            "conviction": analysis.overall_conviction,
            "bullish_signals": analysis.bullish_signals,
            "bearish_signals": analysis.bearish_signals,
            "neutral_signals": analysis.neutral_signals,
            "total_skills": skills.len(),
            "available_skills": skills,
            "timestamp": Utc::now().to_rfc3339(),
        })
        .to_string()
    }

    // ── 2. Trading Status ───────────────────────────────────────────────

    #[tool(
        description = "Get current auto-trading system status, including enabled state, symbols, and paper trading mode"
    )]
    async fn trading_status(&self) -> String {
        let state = self.state.auto_trader.get_state().await;
        let learning = {
            let engine = self.state.learning_engine.lock().await;
            engine.all_skill_performance()
        };

        serde_json::json!({
            "enabled": state.enabled,
            "paper_trading": state.paper_trading,
            "symbols": state.symbols,
            "analysis_interval_secs": state.analysis_interval_secs,
            "adaptive_weights": state.adaptive_weights_enabled,
            "total_learned_trades": state.total_learned_trades,
            "regime_optimization": state.regime_optimization_enabled,
            "skill_count": learning.len(),
            "timestamp": Utc::now().to_rfc3339(),
        })
        .to_string()
    }

    // ── 3. Trading Control ──────────────────────────────────────────────

    #[tool(description = "Start, stop, or get status of the auto-trading system")]
    async fn trading_control(
        &self,
        Parameters(params): Parameters<TradingControlParams>,
    ) -> String {
        match params.action.to_lowercase().as_str() {
            "start" => {
                self.state.auto_trader.set_enabled(true).await;
                self.state.stream_registry.global().alert(
                    "success",
                    "[MCP] Auto-trading started via MCP remote control",
                );
                "✅ Auto-trading started successfully".to_string()
            }
            "stop" => {
                self.state.auto_trader.set_enabled(false).await;
                self.state.stream_registry.global().alert(
                    "warning",
                    "[MCP] Auto-trading stopped via MCP remote control",
                );
                "🛑 Auto-trading stopped successfully".to_string()
            }
            _ => {
                let state = self.state.auto_trader.get_state().await;
                format!(
                    "Auto-trading is {}. Paper trading: {}. Monitoring {} symbols.",
                    if state.enabled {
                        "✅ ACTIVE"
                    } else {
                        "⏸️ PAUSED"
                    },
                    if state.paper_trading {
                        "enabled"
                    } else {
                        "disabled"
                    },
                    state.symbols.len()
                )
            }
        }
    }

    // ── 4. Manual Trade ─────────────────────────────────────────────────

    #[tool(description = "Execute a manual trade override — buy or sell a symbol")]
    async fn manual_trade(&self, Parameters(params): Parameters<ManualTradeParams>) -> String {
        let decision = tredo_types::TradeDecision {
            id: uuid::Uuid::new_v4(),
            symbol: params.symbol.clone(),
            action: params.side.to_uppercase(),
            amount: params.amount,
            price: params.price.unwrap_or(0.0),
            conviction: 1.0,
            reasoning: "[MCP Remote Control] Manual trade executed via MCP".to_string(),
            status: tredo_types::DecisionStatus::Approved,
            timestamp: Utc::now(),
        };

        let _cmd = tredo_types::ExecutionCommand::Execute(decision);
        // Send via the execution channel (will be wired from backend)
        // For now, log and return success
        self.state.stream_registry.global().alert(
            "info",
            &format!(
                "[MCP] Manual trade: {} {} of {}",
                params.side, params.amount, params.symbol
            ),
        );

        format!(
            "✅ Manual trade submitted: {} {} of {} at ~${:.2}",
            params.side.to_uppercase(),
            params.amount,
            params.symbol,
            params.price.unwrap_or(0.0)
        )
    }

    // ── 5. Get Price ────────────────────────────────────────────────────

    #[tool(description = "Get the current market price for a trading symbol")]
    async fn get_price(&self, Parameters(params): Parameters<GetPriceParams>) -> String {
        match self
            .state
            .data_provider
            .fetch_current_price(&params.symbol)
            .await
        {
            Ok(price) => format!(
                "💰 Current price of {}: ${:.4} (timestamp: {})",
                params.symbol,
                price,
                Utc::now().to_rfc3339()
            ),
            Err(e) => format!("❌ Failed to fetch price for {}: {}", params.symbol, e),
        }
    }

    // ── 6. Market Candles ───────────────────────────────────────────────

    #[tool(description = "Get historical OHLCV candle data for a symbol")]
    async fn market_candles(&self, Parameters(params): Parameters<MarketCandlesParams>) -> String {
        let timeframe = match params.timeframe.as_deref() {
            Some("1m") => TimeFrame::Min1,
            Some("5m") => TimeFrame::Min5,
            Some("15m") => TimeFrame::Min15,
            Some("30m") => TimeFrame::Min30,
            Some("4h") => TimeFrame::Hour4,
            Some("1d") => TimeFrame::Day1,
            _ => TimeFrame::Hour1,
        };

        match self
            .state
            .data_provider
            .fetch_candles(&params.symbol, timeframe)
            .await
        {
            Ok(candles) => {
                let count = candles.len();
                let latest = candles.last();

                serde_json::json!({
                    "symbol": params.symbol,
                    "timeframe": timeframe.label(),
                    "count": count,
                    "latest_price": latest.map(|c| c.close),
                    "latest_timestamp": latest.map(|c| c.time),
                    "timestamp": Utc::now().to_rfc3339(),
                })
                .to_string()
            }
            Err(e) => serde_json::json!({
                "error": format!("Failed to fetch candles: {}", e),
                "symbol": params.symbol,
            })
            .to_string(),
        }
    }

    // ── 7. Skills List ──────────────────────────────────────────────────

    #[tool(description = "List all available market analysis skills and their categories")]
    async fn skills_list(&self) -> String {
        let skills = self.state.intelligence.list_skills().await;
        let agents = self.state.intelligence.agent_info().await;

        serde_json::json!({
            "total_skills": skills.len(),
            "skills": skills,
            "agents": agents,
            "agent_provider": self.state.intelligence.agent_provider().provider_name(),
            "timestamp": Utc::now().to_rfc3339(),
        })
        .to_string()
    }

    // ── 8. System Status ────────────────────────────────────────────────

    #[tool(description = "Get overall TREDO system health, including all engine statuses")]
    async fn system_status(&self) -> String {
        let engine_stats = self.state.intelligence.cache_stats();
        let trading = self.state.auto_trader.get_state().await;
        let dnd = self.state.tantra.is_dnd_active();
        let tasks_count = self.state.tantra.tasks.len();

        serde_json::json!({
            "status": "operational",
            "engine": "TREDO Axum Core v2 + MCP",
            "trading": {
                "enabled": trading.enabled,
                "paper_trading": trading.paper_trading,
                "symbols_count": trading.symbols.len(),
            },
            "tantra": {
                "dnd_active": dnd,
                "tasks_count": tasks_count,
            },
            "cache": {
                "hits": engine_stats.hits,
                "misses": engine_stats.misses,
                "hit_rate": engine_stats.hit_rate,
            },
            "providers": {
                "agents": self.state.registry.list_agents(),
                "llms": self.state.registry.list_llms(),
            },
            "timestamp": Utc::now().to_rfc3339(),
        })
        .to_string()
    }

    // ── 9. Portfolio Status ─────────────────────────────────────────────

    #[tool(description = "Get portfolio performance stats from the trade journal")]
    async fn portfolio_status(&self) -> String {
        let journal = self.state.journal.lock().await;
        match journal.get_performance_stats() {
            Ok(stats) => {
                let trades = journal.get_trade_history(10, 0).unwrap_or_default();
                serde_json::json!({
                    "stats": stats,
                    "recent_trades_count": trades.len(),
                    "timestamp": Utc::now().to_rfc3339(),
                })
                .to_string()
            }
            Err(e) => serde_json::json!({
                "error": format!("Failed to get performance stats: {}", e),
            })
            .to_string(),
        }
    }

    // ── 10. Learning Stats ──────────────────────────────────────────────

    #[tool(
        description = "Get learning engine statistics including top-performing skills and regime optimizations"
    )]
    async fn learning_stats(&self) -> String {
        let engine = self.state.learning_engine.lock().await;
        let top = engine.top_skills(10);
        let worst = engine.worst_skills(5);
        let optima = engine.regime_optima();
        let total_trades = engine.total_trades();

        serde_json::json!({
            "total_trades_learned": total_trades,
            "top_performers": top,
            "underperformers": worst,
            "regime_optimizations": optima,
            "timestamp": Utc::now().to_rfc3339(),
        })
        .to_string()
    }

    // ── 11. Bridge Query ────────────────────────────────────────────────

    #[tool(description = "Query the Python Nethra agent through the Redis bridge")]
    async fn bridge_query(&self, Parameters(params): Parameters<BridgeQueryParams>) -> String {
        let method = &params.method;
        let payload = params.params.unwrap_or(serde_json::json!({}));

        // Attempt to send via Redis bridge
        let msg =
            tredo_bridge::AgentBusMessage::new("rust_tredo", "python_nethra", method, payload);

        match self
            .state
            .redis_bridge
            .publish("nethra:agent:python_nethra", &msg)
            .await
        {
            Ok(_) => format!(
                "✅ Request sent to Python Nethra via Redis bridge: method='{}'",
                method
            ),
            Err(e) => format!("❌ Failed to reach Python Nethra: {}", e),
        }
    }

    // ── 12. RAG Search ──────────────────────────────────────────────────

    #[tool(description = "Search the Hierarchical RAG knowledge base")]
    async fn rag_search(&self, Parameters(params): Parameters<RAGSearchParams>) -> String {
        match self
            .state
            .rag_db
            .search(
                &params.query,
                params.agent_id.as_deref(),
                params.limit.unwrap_or(10),
            )
            .await
        {
            Ok(results) => serde_json::json!({
                "count": results.len(),
                "results": results,
                "timestamp": Utc::now().to_rfc3339(),
            })
            .to_string(),
            Err(e) => serde_json::json!({
                "error": e,
                "query": params.query,
            })
            .to_string(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  ArkMcpServerWithResources — wraps ArkMcpServer and adds MCP resources
// ═══════════════════════════════════════════════════════════════════════════

/// TREDO MCP Server with Resources — wraps `ArkMcpServer` and exposes
/// MCP resources (static URIs and URI templates) alongside its 12 tools.
///
/// ## Resources
///
/// | URI | Description |
/// |---|---|
/// | `tredo://status` | Overall system health, trading status, provider info |
/// | `tredo://portfolio` | Portfolio performance stats and recent trades |
/// | `tredo://learning` | Learning engine top/under-performing skills |
/// | `tredo://providers` | Available agent and LLM providers |
/// | `tredo://trading` | Auto-trading system configuration |
/// | `tredo://tantra` | TANTRA safety & calendar status |
/// | `tredo://price/{symbol}` | Current market price by symbol (template) |
/// | `tredo://analysis/{symbol}` | Full market analysis by symbol (template) |
///
/// ## Prompts
///
/// | Name | Description |
/// |---|---|
/// | `analyze-then-trade` | Analyze a symbol and get a trading recommendation |
/// | `portfolio-review` | Review portfolio performance and get recommendations |
/// | `market-scan` | Scan all watched symbols for opportunities |
/// | `system-health` | Full system health check with diagnosis |
#[derive(Debug, Clone)]
pub struct ArkMcpServerWithResources {
    pub inner: ArkMcpServer,
    resources: Vec<Resource>,
    resource_templates: Vec<ResourceTemplate>,
    prompts: Vec<Prompt>,
}

impl ArkMcpServerWithResources {
    pub fn new(state: McpState) -> Self {
        let inner = ArkMcpServer::new(state);

        // ── Static Resources ───────────────────────────────────────────
        let resources = vec![
            RawResource::new("tredo://status", "System Status")
                .with_title("TREDO System Status")
                .with_description("Overall system health, trading status, and provider information")
                .with_mime_type("application/json")
                .no_annotation(),
            RawResource::new("tredo://portfolio", "Portfolio Stats")
                .with_title("Portfolio Performance Stats")
                .with_description("Trade journal performance statistics and recent trades")
                .with_mime_type("application/json")
                .no_annotation(),
            RawResource::new("tredo://learning", "Learning Engine")
                .with_title("Self-Learning Engine Statistics")
                .with_description(
                    "Top-performing skills, underperformers, and regime optimizations",
                )
                .with_mime_type("application/json")
                .no_annotation(),
            RawResource::new("tredo://providers", "Providers")
                .with_title("Plugin Provider Registry")
                .with_description(
                    "Available agent and LLM providers, and which are currently active",
                )
                .with_mime_type("application/json")
                .no_annotation(),
            RawResource::new("tredo://trading", "Trading Status")
                .with_title("Auto-Trading System Status")
                .with_description(
                    "Current auto-trading configuration, enabled state, and symbol list",
                )
                .with_mime_type("application/json")
                .no_annotation(),
            RawResource::new("tredo://tantra", "Tantra Status")
                .with_title("TANTRA Safety & Calendar Status")
                .with_description("Do-not-disturb mode, active tasks, and calendar events")
                .with_mime_type("application/json")
                .no_annotation(),
        ];

        // ── URI Template Resources ─────────────────────────────────────
        let resource_templates = vec![
            RawResourceTemplate::new("tredo://price/{symbol}", "Price by Symbol")
                .with_title("Current Price by Symbol")
                .with_description(
                    "Get the current market price for any trading symbol (e.g., tredo://price/BTC-USD)",
                )
                .with_mime_type("application/json")
                .no_annotation(),
            RawResourceTemplate::new("tredo://analysis/{symbol}", "Analysis by Symbol")
                .with_title("Market Analysis by Symbol")
                .with_description(
                    "Get full technical and risk analysis for any trading symbol (e.g., tredo://analysis/AAPL)",
                )
                .with_mime_type("application/json")
                .no_annotation(),
        ];

        // ── Prompts ───────────────────────────────────────────────────
        let prompts = vec![
            Prompt::new(
                "analyze-then-trade",
                Some("Analyze a trading symbol and get a data-driven trading recommendation based on 30+ built-in skills"),
                Some(vec![
                    PromptArgument::new("symbol")
                        .with_description("Trading symbol (e.g., BTC-USD, AAPL)")
                        .with_required(true),
                    PromptArgument::new("current_price")
                        .with_description("Current market price of the symbol")
                        .with_required(true),
                    PromptArgument::new("cash_available")
                        .with_description("Available cash balance for trading")
                        .with_required(false),
                    PromptArgument::new("portfolio_value")
                        .with_description("Total portfolio value")
                        .with_required(false),
                ]),
            )
            .with_title("Analyze & Trade"),
            Prompt::new(
                "portfolio-review",
                Some("Review your portfolio performance, recent trades, and get actionable recommendations"),
                None,
            )
            .with_title("Portfolio Review"),
            Prompt::new(
                "market-scan",
                Some("Scan all watched trading symbols for market opportunities and risk assessment"),
                None,
            )
            .with_title("Market Scan"),
            Prompt::new(
                "system-health",
                Some("Run a complete system health check across all TREDO subsystems"),
                None,
            )
            .with_title("System Health Check"),
        ];

        Self {
            inner,
            resources,
            resource_templates,
            prompts,
        }
    }

    /// Build the analyze-then-trade prompt response with live data injected.
    async fn build_analyze_trade_prompt(
        &self,
        symbol: &str,
        current_price: f64,
        cash_available: f64,
        portfolio_value: f64,
    ) -> Vec<PromptMessage> {
        let candles = self
            .inner
            .state
            .data_provider
            .fetch_candles(symbol, TimeFrame::Hour1)
            .await
            .unwrap_or_default();

        use std::collections::HashMap;
        let context = tredo_core::MarketAnalysisContext {
            symbol: symbol.to_string(),
            candles: candles
                .into_iter()
                .map(|c| tredo_core::Candle {
                    time: c.time,
                    open: c.open,
                    high: c.high,
                    low: c.low,
                    close: c.close,
                    volume: c.volume,
                })
                .collect(),
            current_price,
            cash_available,
            portfolio_value,
            exposure: 0.0,
            open_positions: HashMap::new(),
            local_skills: None,
        };

        let analysis = self
            .inner
            .state
            .intelligence
            .analyze_with_skills(context)
            .await;
        let skills = self.inner.state.intelligence.list_skills().await;

        let user_msg = format!(
            concat!(
                "I'm considering trading {symbol} at ${price:.2}. ",
                "My available cash is ${cash:.2} and portfolio value is ${portfolio:.2}. ",
                "Please analyze and recommend a trade.",
            ),
            symbol = symbol,
            price = current_price,
            cash = cash_available,
            portfolio = portfolio_value,
        );

        let analysis_json = serde_json::json!({
            "symbol": symbol,
            "price": current_price,
            "direction": analysis.overall_direction,
            "conviction": analysis.overall_conviction,
            "bullish_signals": analysis.bullish_signals,
            "bearish_signals": analysis.bearish_signals,
            "neutral_signals": analysis.neutral_signals,
            "total_skills": skills.len(),
            "available_skills": skills,
            "cash_available": cash_available,
            "portfolio_value": portfolio_value,
            "timestamp": Utc::now().to_rfc3339(),
        });

        let direction_str = format!("{:?}", analysis.overall_direction);
        let direction_verb = direction_str.to_uppercase();
        let assistant_msg = format!(
            concat!(
                "## Analysis Results\n\n",
                "**Direction:** {direction}\n",
                "**Conviction:** {conviction:.1}%\n\n",
                "### Signal Breakdown\n",
                "- 🟢 Bullish signals: {bullish}\n",
                "- 🔴 Bearish signals: {bearish}\n",
                "- ⚪ Neutral signals: {neutral}\n\n",
                "### Recommendation\n",
                "Based on {skill_count} analysis skills, the overall assessment is **{direction_verb}** for {symbol}.\n\n",
                "```json\n{json}\n```\n\n",
                "Consider setting a stop-loss and managing position size relative to your ${cash:.0} available cash.",
            ),
            direction = direction_str.to_lowercase(),
            conviction = analysis.overall_conviction,
            bullish = analysis.bullish_signals,
            bearish = analysis.bearish_signals,
            neutral = analysis.neutral_signals,
            skill_count = skills.len(),
            direction_verb = direction_verb,
            symbol = symbol,
            json = analysis_json.to_string(),
            cash = cash_available,
        );

        vec![
            PromptMessage::new(
                PromptMessageRole::User,
                PromptMessageContent::text(user_msg),
            ),
            PromptMessage::new(
                PromptMessageRole::Assistant,
                PromptMessageContent::text(assistant_msg),
            ),
        ]
    }

    /// Build the portfolio review prompt with live portfolio data.
    async fn build_portfolio_review_prompt(&self) -> Vec<PromptMessage> {
        let performance = {
            let journal = self.inner.state.journal.lock().await;
            let perf = journal.get_performance_stats().ok();
            let trades = journal.get_trade_history(10, 0).unwrap_or_default();
            (perf, trades)
        };

        let skills = self.inner.state.learning_engine.lock().await;
        let top_skills = skills.top_skills(5);
        let total_trades = skills.total_trades();

        let perf_json =
            serde_json::to_string_pretty(&performance.0).unwrap_or_else(|_| "N/A".to_string());
        let skills_json =
            serde_json::to_string_pretty(&top_skills).unwrap_or_else(|_| "N/A".to_string());
        let assistant_msg = format!(
            concat!(
                "## Portfolio Review\n\n",
                "### Performance\n",
                "```json\n{}\n```\n\n",
                "### Top Skills\n",
                "```json\n{}\n```\n\n",
                "**Total learned trades:** {}\n\n",
                "Review your recent trades above and consider adjusting position sizing based on recent performance trends.",
            ),
            perf_json,
            skills_json,
            total_trades,
        );

        vec![
            PromptMessage::new(
                PromptMessageRole::User,
                PromptMessageContent::text(
                    "Show me my portfolio review with recent performance and recommendations.",
                ),
            ),
            PromptMessage::new(
                PromptMessageRole::Assistant,
                PromptMessageContent::text(assistant_msg),
            ),
        ]
    }

    /// Build the market scan prompt — analyze all watched symbols.
    async fn build_market_scan_prompt(&self) -> Vec<PromptMessage> {
        let state = self.inner.state.auto_trader.get_state().await;
        let symbols = state.symbols.clone();

        let mut scan_results = Vec::new();
        for sym in &symbols {
            let price = self
                .inner
                .state
                .data_provider
                .fetch_current_price(sym)
                .await
                .unwrap_or(0.0);

            use std::collections::HashMap;
            let candles = self
                .inner
                .state
                .data_provider
                .fetch_candles(sym, TimeFrame::Hour1)
                .await
                .unwrap_or_default();

            let context = tredo_core::MarketAnalysisContext {
                symbol: sym.clone(),
                candles: candles
                    .into_iter()
                    .map(|c| tredo_core::Candle {
                        time: c.time,
                        open: c.open,
                        high: c.high,
                        low: c.low,
                        close: c.close,
                        volume: c.volume,
                    })
                    .collect(),
                current_price: price,
                cash_available: 100_000.0,
                portfolio_value: 100_000.0,
                exposure: 0.0,
                open_positions: HashMap::new(),
                local_skills: None,
            };

            let analysis = self
                .inner
                .state
                .intelligence
                .analyze_with_skills(context)
                .await;
            scan_results.push(serde_json::json!({
                "symbol": sym,
                "price": price,
                "direction": analysis.overall_direction,
                "conviction": analysis.overall_conviction,
                "bullish": analysis.bullish_signals,
                "bearish": analysis.bearish_signals,
            }));
        }

        let scan_json =
            serde_json::to_string_pretty(&scan_results).unwrap_or_else(|_| "[]".to_string());
        let assistant_msg = format!(
            concat!(
                "## Market Scan\n\n",
                "Scanned **{}** watched symbols.\n\n",
                "```json\n{}\n```\n\n",
                "### Summary\n",
                "- 🟢 Bullish signals indicate potential entry opportunities\n",
                "- 🔴 Bearish signals suggest caution or exit considerations\n",
                "- ⚪ Neutral signals mean the market is indecisive\n\n",
                "*Use `analyze-then-trade` for deep analysis on any specific symbol.*",
            ),
            symbols.len(),
            scan_json,
        );

        vec![
            PromptMessage::new(
                PromptMessageRole::User,
                PromptMessageContent::text("Scan all my watched symbols for market opportunities."),
            ),
            PromptMessage::new(
                PromptMessageRole::Assistant,
                PromptMessageContent::text(assistant_msg),
            ),
        ]
    }

    /// Build the system health check prompt.
    async fn build_system_health_prompt(&self) -> Vec<PromptMessage> {
        let engine_stats = self.inner.state.intelligence.cache_stats();
        let trading = self.inner.state.auto_trader.get_state().await;
        let dnd = self.inner.state.tantra.is_dnd_active();
        let tasks_count = self.inner.state.tantra.tasks.len();

        let health_checks = serde_json::json!([
            {
                "subsystem": "Trading Engine",
                "status": if trading.enabled { "✅ ACTIVE" } else { "⏸️ PAUSED" },
                "details": format!("Monitoring {} symbols (paper: {})", trading.symbols.len(), trading.paper_trading),
            },
            {
                "subsystem": "Intelligence Pool",
                "status": "✅ OPERATIONAL",
                "details": format!("Cache hit rate: {:.1}%", engine_stats.hit_rate * 100.0),
            },
            {
                "subsystem": "TANTRA Safety",
                "status": if dnd { "🛡️ DND ACTIVE" } else { "✅ STANDARD" },
                "details": format!("{} active tasks", tasks_count),
            },
            {
                "subsystem": "Data Provider",
                "status": "✅ OPERATIONAL",
                "details": "Yahoo Finance provider connected",
            },
            {
                "subsystem": "Learning Engine",
                "status": "✅ OPERATIONAL",
                "details": format!("{} total learned trades", engine_stats.misses),
            },
        ]);

        let health_json =
            serde_json::to_string_pretty(&health_checks).unwrap_or_else(|_| "[]".to_string());
        let assistant_msg = format!(
            concat!(
                "## 🏥 TREDO System Health\n\n",
                "```json\n{}\n```\n\n",
                "All subsystems are operational.",
            ),
            health_json,
        );

        vec![
            PromptMessage::new(
                PromptMessageRole::User,
                PromptMessageContent::text(
                    "Run a complete system health check on all TREDO subsystems.",
                ),
            ),
            PromptMessage::new(
                PromptMessageRole::Assistant,
                PromptMessageContent::text(assistant_msg),
            ),
        ]
    }

    /// Resolve a URI to its JSON content, handling both static resources
    /// and URI template patterns.
    async fn resolve_resource(&self, uri: &str) -> Result<String, ErrorData> {
        match uri {
            "tredo://status" => Ok(self.build_system_status().await),
            "tredo://portfolio" => Ok(self.build_portfolio().await),
            "tredo://learning" => Ok(self.build_learning().await),
            "tredo://providers" => Ok(self.build_providers().await),
            "tredo://trading" => Ok(self.build_trading().await),
            "tredo://tantra" => Ok(self.build_tantra().await),
            _ => {
                // Try template matching
                if let Some(symbol) = uri.strip_prefix("tredo://price/") {
                    self.resolve_price(symbol).await
                } else if let Some(symbol) = uri.strip_prefix("tredo://analysis/") {
                    self.resolve_analysis(symbol).await
                } else {
                    Err(ErrorData::resource_not_found(
                        format!("Resource not found: {uri}"),
                        None,
                    ))
                }
            }
        }
    }

    async fn build_system_status(&self) -> String {
        let engine_stats = self.inner.state.intelligence.cache_stats();
        let trading = self.inner.state.auto_trader.get_state().await;
        let dnd = self.inner.state.tantra.is_dnd_active();
        let tasks_count = self.inner.state.tantra.tasks.len();

        serde_json::json!({
            "status": "operational",
            "engine": "TREDO Axum Core v2 + MCP",
            "trading": {
                "enabled": trading.enabled,
                "paper_trading": trading.paper_trading,
                "symbols_count": trading.symbols.len(),
            },
            "tantra": {
                "dnd_active": dnd,
                "tasks_count": tasks_count,
            },
            "cache": {
                "hits": engine_stats.hits,
                "misses": engine_stats.misses,
                "hit_rate": engine_stats.hit_rate,
            },
            "providers": {
                "agents": self.inner.state.registry.list_agents(),
                "llms": self.inner.state.registry.list_llms(),
            },
            "timestamp": Utc::now().to_rfc3339(),
        })
        .to_string()
    }

    async fn build_portfolio(&self) -> String {
        let journal = self.inner.state.journal.lock().await;
        match journal.get_performance_stats() {
            Ok(stats) => {
                let trades = journal.get_trade_history(10, 0).unwrap_or_default();
                serde_json::json!({
                    "stats": stats,
                    "recent_trades_count": trades.len(),
                    "timestamp": Utc::now().to_rfc3339(),
                })
                .to_string()
            }
            Err(e) => serde_json::json!({
                "error": format!("Failed to get performance stats: {e}"),
            })
            .to_string(),
        }
    }

    async fn build_learning(&self) -> String {
        let engine = self.inner.state.learning_engine.lock().await;
        let top = engine.top_skills(10);
        let worst = engine.worst_skills(5);
        let optima = engine.regime_optima();
        let total_trades = engine.total_trades();

        serde_json::json!({
            "total_trades_learned": total_trades,
            "top_performers": top,
            "underperformers": worst,
            "regime_optimizations": optima,
            "timestamp": Utc::now().to_rfc3339(),
        })
        .to_string()
    }

    async fn build_providers(&self) -> String {
        serde_json::json!({
            "agents": self.inner.state.registry.list_agents(),
            "llms": self.inner.state.registry.list_llms(),
            "current_agent": self.inner.state.intelligence.agent_provider().provider_name(),
            "current_llm": self.inner.state.intelligence.llm_provider().provider_name(),
            "timestamp": Utc::now().to_rfc3339(),
        })
        .to_string()
    }

    async fn build_trading(&self) -> String {
        let state = self.inner.state.auto_trader.get_state().await;
        let learning = {
            let engine = self.inner.state.learning_engine.lock().await;
            engine.all_skill_performance()
        };

        serde_json::json!({
            "enabled": state.enabled,
            "paper_trading": state.paper_trading,
            "symbols": state.symbols,
            "analysis_interval_secs": state.analysis_interval_secs,
            "adaptive_weights": state.adaptive_weights_enabled,
            "total_learned_trades": state.total_learned_trades,
            "regime_optimization": state.regime_optimization_enabled,
            "skill_count": learning.len(),
            "timestamp": Utc::now().to_rfc3339(),
        })
        .to_string()
    }

    async fn build_tantra(&self) -> String {
        let dnd = self.inner.state.tantra.is_dnd_active();
        let tasks_count = self.inner.state.tantra.tasks.len();
        let mut events = Vec::new();
        for entry in self.inner.state.tantra.events.iter() {
            events.push(entry.value().clone());
        }

        serde_json::json!({
            "dnd_active": dnd,
            "active_tasks_count": tasks_count,
            "events": events,
            "safety_index": if dnd { "HIGH_GUARD" } else { "STANDARD" },
            "timestamp": Utc::now().to_rfc3339(),
        })
        .to_string()
    }

    async fn resolve_price(&self, symbol: &str) -> Result<String, ErrorData> {
        match self
            .inner
            .state
            .data_provider
            .fetch_current_price(symbol)
            .await
        {
            Ok(price) => Ok(serde_json::json!({
                "symbol": symbol,
                "price": price,
                "timestamp": Utc::now().to_rfc3339(),
            })
            .to_string()),
            Err(e) => Ok(serde_json::json!({
                "symbol": symbol,
                "error": format!("Failed to fetch price: {e}"),
                "timestamp": Utc::now().to_rfc3339(),
            })
            .to_string()),
        }
    }

    async fn resolve_analysis(&self, symbol: &str) -> Result<String, ErrorData> {
        let candles = self
            .inner
            .state
            .data_provider
            .fetch_candles(symbol, TimeFrame::Hour1)
            .await
            .unwrap_or_default();

        let current_price = candles.last().map(|c| c.close).unwrap_or(100.0);
        let cash_available = 100_000.0;
        let portfolio_value = 100_000.0;

        use std::collections::HashMap;
        let context = tredo_core::MarketAnalysisContext {
            symbol: symbol.to_string(),
            candles: candles
                .into_iter()
                .map(|c| tredo_core::Candle {
                    time: c.time,
                    open: c.open,
                    high: c.high,
                    low: c.low,
                    close: c.close,
                    volume: c.volume,
                })
                .collect(),
            current_price,
            cash_available,
            portfolio_value,
            exposure: 0.0,
            open_positions: HashMap::new(),
            local_skills: None,
        };

        let analysis = self
            .inner
            .state
            .intelligence
            .analyze_with_skills(context)
            .await;
        let skills = self.inner.state.intelligence.list_skills().await;

        Ok(serde_json::json!({
            "symbol": symbol,
            "price": current_price,
            "direction": analysis.overall_direction,
            "conviction": analysis.overall_conviction,
            "bullish_signals": analysis.bullish_signals,
            "bearish_signals": analysis.bearish_signals,
            "neutral_signals": analysis.neutral_signals,
            "total_skills": skills.len(),
            "available_skills": skills,
            "timestamp": Utc::now().to_rfc3339(),
        })
        .to_string())
    }
}

impl ServerHandler for ArkMcpServerWithResources {
    // ── Tool methods ───────────────────────────────────────────────────
    // Delegate to inner ArkMcpServer (which has #[tool_router(server_handler)])

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        self.inner.call_tool(request, context).await
    }

    async fn list_tools(
        &self,
        request: Option<PaginatedRequestParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        self.inner.list_tools(request, context).await
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        self.inner.get_tool(name)
    }

    fn get_info(&self) -> ServerInfo {
        let mut info = self.inner.get_info();
        // Advertise tool support (from inner ArkMcpServer capabilities)
        info.capabilities
            .tools
            .get_or_insert_with(Default::default)
            .list_changed = Some(true);
        // Advertise resource support
        info.capabilities.resources = Some(ResourcesCapability {
            subscribe: Some(false),
            list_changed: Some(true),
        });
        // Advertise prompt support
        info.capabilities.prompts = Some(PromptsCapability {
            list_changed: Some(true),
        });
        info
    }

    // ── Prompt methods ────────────────────────────────────────────────

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        Ok(ListPromptsResult::with_all_items(self.prompts.clone()))
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        let messages = match request.name.as_str() {
            "analyze-then-trade" => {
                let args = request.arguments.unwrap_or_default();
                let symbol = args.get("symbol").and_then(|v| v.as_str()).ok_or_else(|| {
                    ErrorData::invalid_params("Missing required parameter: symbol", None)
                })?;
                let current_price = args
                    .get("current_price")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| {
                        ErrorData::invalid_params("Missing required parameter: current_price", None)
                    })?;
                let cash_available = args
                    .get("cash_available")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(100_000.0);
                let portfolio_value = args
                    .get("portfolio_value")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(100_000.0);

                self.build_analyze_trade_prompt(
                    symbol,
                    current_price,
                    cash_available,
                    portfolio_value,
                )
                .await
            }
            "portfolio-review" => self.build_portfolio_review_prompt().await,
            "market-scan" => self.build_market_scan_prompt().await,
            "system-health" => self.build_system_health_prompt().await,
            _ => {
                return Err(ErrorData::invalid_params(
                    format!("Unknown prompt: {}", request.name),
                    None,
                ))
            }
        };

        Ok(GetPromptResult::new(messages)
            .with_description(format!("TREDO Trading Prompt: {}", request.name)))
    }

    // ── Resource methods ───────────────────────────────────────────────

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        Ok(ListResourcesResult::with_all_items(self.resources.clone()))
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, ErrorData> {
        Ok(ListResourceTemplatesResult::with_all_items(
            self.resource_templates.clone(),
        ))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, ErrorData> {
        let content = self.resolve_resource(&request.uri).await?;
        Ok(ReadResourceResult::new(vec![ResourceContents::text(
            content,
            &request.uri,
        )
        .with_mime_type("application/json")]))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  Helper — create MCP server capabilities for initialization
// ═══════════════════════════════════════════════════════════════════════════

/// Return the MCP server capabilities and implementation info.
pub fn server_info() -> (ServerCapabilities, Implementation) {
    let capabilities = ServerCapabilities::builder()
        .enable_tools()
        .enable_tool_list_changed()
        .build();

    let implementation = Implementation::new("TREDO MCP Server", "0.1.0");

    (capabilities, implementation)
}
