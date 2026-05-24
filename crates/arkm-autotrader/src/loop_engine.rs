use crate::config::AutoTradingConfig;
use crate::regime::{MarketRegime, RegimeDetector};
use arkm_data::{MarketDataProvider, TimeFrame, YahooFinanceProvider};
use arkm_journal::{DecisionRecord, PerformanceStats, TradeJournal, TradeRecord};
use arkm_learning::LearningEngine;
use arkm_core::{AgentProvider, AggregatedAnalysis, MarketAnalysisContext, SignalDirection};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Action recommended by the trading loop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeAction {
    Buy(String, f64, f64),  // (symbol, price, quantity)
    Sell(String, f64, f64), // (symbol, price, quantity)
    Hold(String),           // No action
    Skip(String, String),   // (symbol, reason)
}

/// Outcome of a single decision cycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionOutcome {
    pub symbol: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub regime: MarketRegime,
    pub action: TradeAction,
    pub conviction: f64,
    pub bullish_signals: u32,
    pub bearish_signals: u32,
    pub neutral_signals: u32,
    pub summary: String,
    /// Learning engine trade ID, for linking open/close (None for Hold/Skip)
    pub learning_trade_id: Option<String>,
}

/// Overall state of the auto-trading system (includes config)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingState {
    // Runtime state
    pub enabled: bool,
    pub paper_trading: bool,
    pub symbols: Vec<String>,
    pub analysis_interval_secs: u64,
    pub last_analysis: Option<chrono::DateTime<chrono::Utc>>,
    pub next_analysis: Option<chrono::DateTime<chrono::Utc>>,
    pub last_outcomes: Vec<DecisionOutcome>,
    pub open_positions: Vec<String>,
    pub current_drawdown_pct: f64,
    pub balance: f64,
    pub performance: Option<PerformanceStats>,

    // Config (merged into state to avoid race conditions)
    pub min_conviction: f64,
    pub min_signals_required: u32,
    pub max_positions: usize,
    pub max_risk_per_trade_pct: f64,
    pub max_drawdown_pct: f64,
    pub trailing_stop_enabled: bool,
    pub trailing_stop_pct: f64,

    // Self-learning state
    pub adaptive_weights_enabled: bool,
    pub total_learned_trades: u64,
    pub learning_skill_count: usize,
    pub regime_optimization_enabled: bool,
}

/// The autonomous trading loop with integrated self-learning
pub struct AutoTradingLoop {
    data_provider: Arc<YahooFinanceProvider>,
    agent: Arc<dyn AgentProvider>,
    journal: Arc<Mutex<TradeJournal>>,
    state: Arc<Mutex<TradingState>>,
    regime_detector: RegimeDetector,
    learning_engine: Arc<Mutex<LearningEngine>>,
    /// Maps journal trade IDs to learning engine trade IDs
    trade_id_map: Arc<Mutex<HashMap<String, String>>>,
}

impl AutoTradingLoop {
    pub fn new(
        config: AutoTradingConfig,
        data_provider: Arc<YahooFinanceProvider>,
        agent: Arc<dyn AgentProvider>,
        journal: Arc<Mutex<TradeJournal>>,
        learning_engine: Arc<Mutex<LearningEngine>>,
    ) -> Self {
        let state = TradingState {
            enabled: config.enabled,
            paper_trading: config.paper_trading,
            symbols: config.symbols,
            analysis_interval_secs: config.analysis_interval_secs,
            last_analysis: None,
            next_analysis: None,
            last_outcomes: Vec::new(),
            open_positions: Vec::new(),
            current_drawdown_pct: 0.0,
            balance: config.paper_balance,
            performance: None,
            min_conviction: config.min_conviction,
            min_signals_required: config.min_signals_required,
            max_positions: config.max_positions,
            max_risk_per_trade_pct: config.max_risk_per_trade_pct,
            max_drawdown_pct: config.max_drawdown_pct,
            trailing_stop_enabled: config.trailing_stop_enabled,
            trailing_stop_pct: config.trailing_stop_pct,
            // Self-learning defaults
            adaptive_weights_enabled: true,
            total_learned_trades: 0,
            learning_skill_count: 0,
            regime_optimization_enabled: true,
        };

        Self {
            data_provider,
            agent,
            journal,
            state: Arc::new(Mutex::new(state)),
            regime_detector: RegimeDetector::new(),
            learning_engine,
            trade_id_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get a snapshot of the current trading state
    pub async fn get_state(&self) -> TradingState {
        self.state.lock().await.clone()
    }

    /// Get a reference to the learning engine
    pub fn learning_engine(&self) -> &Arc<Mutex<LearningEngine>> {
        &self.learning_engine
    }

    /// Update config at runtime
    pub async fn update_config(&self, new_config: AutoTradingConfig) {
        let mut guard = self.state.lock().await;
        guard.enabled = new_config.enabled;
        guard.paper_trading = new_config.paper_trading;
        guard.symbols = new_config.symbols;
        guard.analysis_interval_secs = new_config.analysis_interval_secs;
        guard.min_conviction = new_config.min_conviction;
        guard.min_signals_required = new_config.min_signals_required;
        guard.max_positions = new_config.max_positions;
        guard.max_risk_per_trade_pct = new_config.max_risk_per_trade_pct;
        guard.max_drawdown_pct = new_config.max_drawdown_pct;
        guard.trailing_stop_enabled = new_config.trailing_stop_enabled;
        guard.trailing_stop_pct = new_config.trailing_stop_pct;
    }

    /// Toggle only the enabled flag (preserves all other config)
    pub async fn set_enabled(&self, enabled: bool) {
        let mut guard = self.state.lock().await;
        guard.enabled = enabled;
    }

    /// Start the autonomous trading loop (runs forever)
    pub async fn run(&self) {
        println!("[AutoTradingLoop] Starting autonomous trading loop with self-learning engine...");

        // Register all agent skills with the learning engine on startup
        {
            let skill_names = self.agent.list_skills().await;
            let mut engine = self.learning_engine.lock().await;
            for (i, name) in skill_names.iter().enumerate() {
                let skill_id = format!("skill_{}", i);
                let base_weight = if name.contains("Risk") { 0.8 }
                    else if name.contains("Portfolio") { 0.6 }
                    else { 1.0 };
                engine.register_skill(&skill_id, name, base_weight);
            }
            let count = skill_names.len();
            println!("[AutoTradingLoop] Registered {} skills with learning engine", count);
        }

        loop {
            let should_run = self.state.lock().await.enabled;

            if should_run {
                let now = Utc::now();
                let interval_secs = {
                    let guard = self.state.lock().await;
                    guard.analysis_interval_secs
                };

                {
                    let mut guard = self.state.lock().await;
                    guard.last_analysis = Some(now);
                    guard.next_analysis = Some(now + chrono::Duration::seconds(interval_secs as i64));
                }

                // Clone symbols to avoid holding the lock during analysis
                let symbols = self.state.lock().await.symbols.clone();

                for symbol in &symbols {
                    let outcome = self.analyze_symbol(symbol).await;
                    self.execute_action(&outcome).await;

                    let mut guard = self.state.lock().await;
                    guard.last_outcomes.push(outcome.clone());
                    if guard.last_outcomes.len() > 100 {
                        guard.last_outcomes.remove(0);
                    }
                }

        // Sync learning engine weights back to the agent provider
        self.sync_learning_to_agent().await;

                // Update self-learning stats in state
                {
                    let mut guard = self.state.lock().await;
                    let engine = self.learning_engine.lock().await;
                    guard.total_learned_trades = engine.total_trades();
                    guard.learning_skill_count = engine.all_skill_performance().len();
                }

                // Update performance stats from journal
                let journal = self.journal.lock().await;
                if let Ok(stats) = journal.get_performance_stats() {
                    let mut guard = self.state.lock().await;
                    guard.performance = Some(stats);
                }

                println!(
                    "[AutoTradingLoop] Cycle complete. Sleeping for {}s...",
                    interval_secs
                );

                tokio::time::sleep(tokio::time::Duration::from_secs(interval_secs)).await;
            } else {
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            }
        }
    }

    /// Analyze a single symbol
    async fn analyze_symbol(&self, symbol: &str) -> DecisionOutcome {
        // 1. Fetch market data
        let candles = self
            .data_provider
            .fetch_candles(symbol, TimeFrame::Hour1)
            .await
            .unwrap_or_default();

        let all_candles = if candles.is_empty() {
            // Fallback to 5m
            self.data_provider
                .fetch_candles(symbol, TimeFrame::Min5)
                .await
                .unwrap_or_default()
        } else {
            candles
        };

        if all_candles.is_empty() {
            return DecisionOutcome {
                symbol: symbol.to_string(),
                timestamp: Utc::now(),
                regime: MarketRegime::Ranging,
                action: TradeAction::Skip(symbol.to_string(), "No market data available".to_string()),
                conviction: 0.0,
                bullish_signals: 0,
                bearish_signals: 0,
                neutral_signals: 0,
                summary: "SKIP — No data from Yahoo Finance".to_string(),
                learning_trade_id: None,
            };
        }

        // 2. Detect market regime
        let regime = self.regime_detector.detect(&all_candles);

        // 3. Get current price
        let current_price = all_candles.last().map(|c| c.close).unwrap_or(0.0);

        // 4. Read state for balance, config, and learning params
        let (balance, min_conviction, min_signals, max_risk_pct, _adaptive_enabled, regime_opt_enabled) = {
            let guard = self.state.lock().await;
            (
                guard.balance,
                guard.min_conviction,
                guard.min_signals_required,
                guard.max_risk_per_trade_pct,
                guard.adaptive_weights_enabled,
                guard.regime_optimization_enabled,
            )
        };

        // 5. Get learning-adjusted conviction threshold
        let learning_conviction = if regime_opt_enabled {
            let engine = self.learning_engine.lock().await;
            engine.suggested_conviction(regime.label())
        } else {
            min_conviction
        };
        let effective_min_conviction = learning_conviction.max(min_conviction);

        // 6. Run agent skills analysis (plugged via AgentProvider trait)
        let context = MarketAnalysisContext {
            symbol: symbol.to_string(),
            candles: all_candles.clone(),
            current_price,
            cash_available: balance,
            portfolio_value: balance,
            exposure: 0.0,
            open_positions: HashMap::new(),
        };

        let analysis = self.agent.analyze_market(&context).await.unwrap_or_else(|e| {
            println!("[AutoTradingLoop] Agent analysis error: {:?}", e);
            AggregatedAnalysis {
                symbol: symbol.to_string(),
                current_price,
                signals: vec![],
                overall_conviction: 0.0,
                overall_direction: SignalDirection::Neutral,
                bullish_signals: 0,
                bearish_signals: 0,
                neutral_signals: 0,
                timestamp: chrono::Utc::now(),
            }
        });

        // 7. Decide action (using learning-adjusted conviction)
        let action = self.decide_action(
            &analysis,
            &regime,
            symbol,
            current_price,
            effective_min_conviction,
            min_signals,
            balance,
            max_risk_pct,
        );

        // 8. Record the decision
        let decision = DecisionRecord {
            id: Uuid::new_v4().to_string(),
            symbol: symbol.to_string(),
            timestamp: Utc::now(),
            overall_conviction: analysis.overall_conviction,
            overall_direction: format!("{:?}", analysis.overall_direction),
            market_regime: regime.label().to_string(),
            action_taken: match &action {
                TradeAction::Buy(_, _, _) => "BUY".to_string(),
                TradeAction::Sell(_, _, _) => "SELL".to_string(),
                TradeAction::Hold(_) => "HOLD".to_string(),
                TradeAction::Skip(_, _) => "SKIP".to_string(),
            },
            reason: format!(
                "Conviction: {:.2}, Regime: {}, Bullish: {}, Bearish: {} (Learning-adjusted threshold: {:.2})",
                analysis.overall_conviction,
                regime.label(),
                analysis.bullish_signals,
                analysis.bearish_signals,
                effective_min_conviction,
            ),
            bullish_signals: analysis.bullish_signals,
            bearish_signals: analysis.bearish_signals,
            neutral_signals: analysis.neutral_signals,
        };

        let journal = self.journal.lock().await;
        let _ = journal.record_decision(&decision);

        // 9. Record signals with learning engine if this is a trade action
        let learning_trade_id = if matches!(action, TradeAction::Buy(_, _, _) | TradeAction::Sell(_, _, _)) {
            let trade_id = Uuid::new_v4().to_string();
            let mut engine = self.learning_engine.lock().await;
            engine.open_trade(
                &trade_id,
                symbol,
                analysis.signals.clone(),
                &context,
                regime.label(),
            );
            Some(trade_id)
        } else {
            None
        };

        let summary = match &action {
            TradeAction::Buy(_, _, _) => format!(
                "BUY {:.2} — Conviction {:.1}% (learned threshold {:.1}%) | {} Bullish, {} Bearish, {} Neutral | Regime: {}",
                current_price,
                analysis.overall_conviction * 100.0,
                effective_min_conviction * 100.0,
                analysis.bullish_signals,
                analysis.bearish_signals,
                analysis.neutral_signals,
                regime.label(),
            ),
            TradeAction::Sell(_, _, _) => format!(
                "SELL {:.2} — Conviction {:.1}% | {} Bullish, {} Bearish, {} Neutral | Regime: {}",
                current_price,
                analysis.overall_conviction * 100.0,
                analysis.bullish_signals,
                analysis.bearish_signals,
                analysis.neutral_signals,
                regime.label(),
            ),
            TradeAction::Hold(_) => format!(
                "HOLD @ {:.2} — Conviction {:.1}% (needs {:.1}%) | Regime: {}",
                current_price,
                analysis.overall_conviction * 100.0,
                effective_min_conviction * 100.0,
                regime.label(),
            ),
            TradeAction::Skip(_, reason) => {
                return DecisionOutcome {
                    symbol: symbol.to_string(),
                    timestamp: Utc::now(),
                    regime,
                    action: TradeAction::Skip(symbol.to_string(), reason.clone()),
                    conviction: analysis.overall_conviction,
                    bullish_signals: analysis.bullish_signals,
                    bearish_signals: analysis.bearish_signals,
                    neutral_signals: analysis.neutral_signals,
                    summary: format!("SKIP — {} (learned threshold: {:.1}%)", reason, effective_min_conviction * 100.0),
                    learning_trade_id: None,
                };
            }
        };

        DecisionOutcome {
            symbol: symbol.to_string(),
            timestamp: Utc::now(),
            regime,
            action,
            conviction: analysis.overall_conviction,
            bullish_signals: analysis.bullish_signals,
            bearish_signals: analysis.bearish_signals,
            neutral_signals: analysis.neutral_signals,
            summary,
            learning_trade_id,
        }
    }

    /// Decide whether to buy, sell, or hold
    #[allow(clippy::too_many_arguments)]
    fn decide_action(
        &self,
        analysis: &AggregatedAnalysis,
        regime: &MarketRegime,
        symbol: &str,
        current_price: f64,
        min_conviction: f64,
        min_signals: u32,
        current_balance: f64,
        max_risk_pct: f64,
    ) -> TradeAction {
        if analysis.signals.len() < min_signals as usize {
            return TradeAction::Skip(
                symbol.to_string(),
                format!("Only {} signals (need {})", analysis.signals.len(), min_signals),
            );
        }

        if analysis.overall_conviction.abs() < min_conviction {
            return TradeAction::Skip(
                symbol.to_string(),
                format!(
                    "Conviction {:.2} below threshold {:.2}",
                    analysis.overall_conviction, min_conviction
                ),
            );
        }

        // Apply regime-based risk adjustment
        let regime_mult = regime.risk_multiplier();
        let adjusted_conviction = analysis.overall_conviction * regime_mult;

        if adjusted_conviction > min_conviction {
            let risk_amount = current_balance * (max_risk_pct / 100.0);
            let quantity = if current_price > 0.0 {
                risk_amount / current_price
            } else {
                0.0
            };

            if quantity <= 0.0 {
                return TradeAction::Skip(symbol.to_string(), "Zero quantity calculated".to_string());
            }

            TradeAction::Buy(symbol.to_string(), current_price, quantity)
        } else if adjusted_conviction < -min_conviction {
            TradeAction::Sell(symbol.to_string(), current_price, 0.0)
        } else {
            TradeAction::Hold(symbol.to_string())
        }
    }

    /// Execute the decided action
    async fn execute_action(&self, outcome: &DecisionOutcome) {
        match &outcome.action {
            TradeAction::Buy(symbol, price, quantity) => {
                let mut guard = self.state.lock().await;

                if guard.open_positions.len() >= guard.max_positions {
                    println!("[AutoTradingLoop] Max positions reached. Skipping buy for {}", symbol);
                    return;
                }

                let cost = price * quantity;
                if cost > guard.balance {
                    println!(
                        "[AutoTradingLoop] Insufficient balance for {} buy: need ${:.2}, have ${:.2}",
                        symbol, cost, guard.balance
                    );
                    return;
                }

                guard.balance -= cost;
                guard.open_positions.push(symbol.clone());

                let journal = self.journal.lock().await;
                let journal_trade_id = Uuid::new_v4().to_string();

                // Store mapping from journal trade ID to learning engine trade ID
                if let Some(learning_id) = &outcome.learning_trade_id {
                    let mut map = self.trade_id_map.lock().await;
                    map.insert(journal_trade_id.clone(), learning_id.clone());
                    drop(map);
                }

                let trade = TradeRecord {
                    id: journal_trade_id.clone(),
                    symbol: symbol.clone(),
                    side: "BUY".to_string(),
                    entry_price: *price,
                    exit_price: None,
                    quantity: *quantity,
                    pnl: None,
                    pnl_pct: None,
                    conviction_at_entry: outcome.conviction,
                    entry_reasoning: outcome.summary.clone(),
                    exit_reasoning: None,
                    market_regime: outcome.regime.label().to_string(),
                    strategies_used: "auto_trade_with_learning".to_string(),
                    open_time: Utc::now(),
                    close_time: None,
                    is_open: true,
                };
                let _ = journal.open_trade(&trade);

                println!(
                    "[AutoTradingLoop] ✅ BUY {} {:.4} @ ${:.2} (Total: ${:.2}, Balance: ${:.2}, Learned threshold: {:.1}%)",
                    symbol, quantity, price, cost, guard.balance,
                    outcome.conviction * 100.0,
                );
            }
            TradeAction::Sell(symbol, price, _) => {
                let mut guard = self.state.lock().await;

                if let Some(pos) = guard.open_positions.iter().position(|s| s == symbol) {
                    guard.open_positions.remove(pos);
                    let quantity_fallback = 0.1;

                    let journal = self.journal.lock().await;
                    if let Ok(open_trades) = journal.get_open_trades() {
                        if let Some(open_trade) = open_trades.iter().find(|t| t.symbol == *symbol) {
                            let pnl = (*price - open_trade.entry_price) * open_trade.quantity;
                            let pnl_pct = if open_trade.entry_price > 0.0 {
                                ((*price - open_trade.entry_price) / open_trade.entry_price) * 100.0
                            } else {
                                0.0
                            };
                            guard.balance += *price * open_trade.quantity + pnl;

                            // Look up the learning engine trade ID from the mapping
                            let learning_trade_id = {
                                let map = self.trade_id_map.lock().await;
                                map.get(&open_trade.id).cloned()
                            };

                            // Record trade outcome in learning engine
                            let mut engine = self.learning_engine.lock().await;
                            // Use the learning trade ID if available, otherwise fall back to journal trade ID
                            let lookup_id = learning_trade_id.as_deref().unwrap_or(&open_trade.id);
                            engine.close_trade(
                                lookup_id,
                                symbol,
                                open_trade.entry_price,
                                *price,
                                pnl_pct,
                                outcome.regime.label(),
                                outcome.conviction,
                            );
                            drop(engine);

                            let _ = journal.close_trade(&open_trade.id, *price, &outcome.summary);

                            // Clean up the mapping
                            let mut map = self.trade_id_map.lock().await;
                            map.remove(&open_trade.id);
                        } else {
                            guard.balance += *price * quantity_fallback;
                        }
                    } else {
                        guard.balance += *price * quantity_fallback;
                    }

                    println!(
                        "[AutoTradingLoop] ✅ SELL {} @ ${:.2}. Balance: ${:.2}",
                        symbol, price, guard.balance
                    );
                } else {
                    println!("[AutoTradingLoop] No position to sell for {}", symbol);
                }
            }
            TradeAction::Hold(_) | TradeAction::Skip(_, _) => {}
        }
    }

    /// Sync learning engine's adaptive weights back to the agent provider
    async fn sync_learning_to_agent(&self) {
        let engine = self.learning_engine.lock().await;
        if !self.state.lock().await.adaptive_weights_enabled {
            return;
        }

        // Group skill performances by category for sub-agent weight updates
        let mut technical_weights: Vec<f64> = Vec::new();
        let mut risk_weights: Vec<f64> = Vec::new();
        let mut portfolio_weights: Vec<f64> = Vec::new();
        let mut market_data_weights: Vec<f64> = Vec::new();

        for perf in engine.all_skill_performance() {
            let skill_name_lower = perf.skill_name.to_lowercase();
            if skill_name_lower.contains("technical analysis") || skill_name_lower.contains("advanced")
                || skill_name_lower.contains("rsi") || skill_name_lower.contains("macd")
                || skill_name_lower.contains("bollinger") || skill_name_lower.contains("sma")
                || skill_name_lower.contains("ema") || skill_name_lower.contains("volume")
                || skill_name_lower.contains("support") || skill_name_lower.contains("ichimoku")
                || skill_name_lower.contains("adx") || skill_name_lower.contains("trend")
                || skill_name_lower.contains("parabolic") || skill_name_lower.contains("keltner")
                || skill_name_lower.contains("aroon") || skill_name_lower.contains("pivot")
                || skill_name_lower.contains("chandelier") || skill_name_lower.contains("williams")
                || skill_name_lower.contains("obv") || skill_name_lower.contains("chaikin")
                || skill_name_lower.contains("stochastic") || skill_name_lower.contains("donchian")
                || skill_name_lower.contains("heikin") || skill_name_lower.contains("market struct")
                || skill_name_lower.contains("cypher")
            {
                technical_weights.push(perf.adjusted_weight);
            } else if skill_name_lower.contains("portfolio") || skill_name_lower.contains("diversification")
                || skill_name_lower.contains("correlation") || skill_name_lower.contains("health")
            {
                portfolio_weights.push(perf.adjusted_weight);
            } else if skill_name_lower.contains("risk") || skill_name_lower.contains("position sizing")
                || skill_name_lower.contains("value at risk") || skill_name_lower.contains("exposure")
                || skill_name_lower.contains("volatility")
            {
                risk_weights.push(perf.adjusted_weight);
            } else {
                market_data_weights.push(perf.adjusted_weight);
            }
        }

        // Compute average weight for each sub-agent, normalized against base
        let avg = |weights: &[f64]| -> f64 {
            if weights.is_empty() { 0.25 }
            else { weights.iter().sum::<f64>() / weights.len() as f64 }
        };

        let scale = |raw: f64| -> f64 { (raw * 0.3 + 0.2).clamp(0.05, 0.80) };

        let tech_weight = scale(avg(&technical_weights));
        let risk_weight = scale(avg(&risk_weights));
        let port_weight = scale(avg(&portfolio_weights));
        let mkt_weight = scale(avg(&market_data_weights));

        let sum = tech_weight + risk_weight + port_weight + mkt_weight;
        if sum > 0.0 {
            self.agent.update_weight("technical_analyst", tech_weight / sum).await;
            self.agent.update_weight("risk_manager", risk_weight / sum).await;
            self.agent.update_weight("portfolio_manager", port_weight / sum).await;
            self.agent.update_weight("market_data_agent", mkt_weight / sum).await;
        }
    }
}
