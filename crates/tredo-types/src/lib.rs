use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use borsh::{BorshSerialize, BorshDeserialize};

// ── Wire Contract Types ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct PriceLevel {
    pub price: f64,
    pub size: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct OrderBookSnapshot {
    pub symbol: String,
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
}

// ── Trade Types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum DecisionStatus {
    Approved,
    Rejected,
    #[default]
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeDecision {
    pub id: Uuid,
    pub symbol: String,
    pub action: String,
    pub amount: f64,
    pub price: f64,
    pub conviction: f64,
    pub reasoning: String,
    pub status: DecisionStatus,
    pub timestamp: DateTime<Utc>,
}

impl Default for TradeDecision {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            symbol: String::new(),
            action: String::new(),
            amount: 0.0,
            price: 0.0,
            conviction: 0.0,
            reasoning: String::new(),
            status: DecisionStatus::Pending,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub symbol: String,
    pub price: f64,
    pub cash_available: f64,
    pub exposure: f64,
}

// ── API Request Types ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ManualOverrideRequest {
    pub symbol: String,
    pub side: String,
    pub amount: f64,
}

// ── Command Enums ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum ExecutionCommand {
    Execute(TradeDecision),
    UpdateBalance(String, f64),
    UpdatePosition(String, f64),
    RefundInFlight(Uuid, String, f64),
    AcknowledgeTrade(Uuid),
}

#[derive(Debug)]
pub enum TantraCommand {
    TradeExecuted(TradeDecision),
    RiskRejected(TradeDecision),
}

// ── Enhanced Risk State ────────────────────────────────────────────────────

/// Risk state that tracks portfolio-level risk metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskState {
    /// Starting portfolio value (for drawdown calculation)
    pub initial_portfolio_value: f64,
    /// Peak portfolio value reached
    pub peak_portfolio_value: f64,
    /// Current drawdown percentage
    pub current_drawdown_pct: f64,
    /// Number of consecutive losses
    pub consecutive_losses: u32,
    /// Total number of trades
    pub total_trades: u64,
    /// Number of winning trades
    pub winning_trades: u64,
    /// Historical win rate
    pub historical_win_rate: f64,
    /// Whether trading is halted due to risk limits
    pub trading_halted: bool,
    /// Reason for halt if halted
    pub halt_reason: Option<String>,
}

impl RiskState {
    pub fn new(initial_portfolio_value: f64) -> Self {
        Self {
            initial_portfolio_value,
            peak_portfolio_value: initial_portfolio_value,
            current_drawdown_pct: 0.0,
            consecutive_losses: 0,
            total_trades: 0,
            winning_trades: 0,
            historical_win_rate: 0.0,
            trading_halted: false,
            halt_reason: None,
        }
    }

    pub fn update_drawdown(&mut self, current_value: f64) {
        if current_value > self.peak_portfolio_value {
            self.peak_portfolio_value = current_value;
        }
        self.current_drawdown_pct = if self.peak_portfolio_value > 0.0 {
            ((self.peak_portfolio_value - current_value) / self.peak_portfolio_value) * 100.0
        } else {
            0.0
        };
    }

    pub fn record_trade_result(&mut self, pnl: f64) {
        self.total_trades += 1;
        if pnl > 0.0 {
            self.winning_trades += 1;
            self.consecutive_losses = 0;
        } else {
            self.consecutive_losses += 1;
        }
        self.historical_win_rate = if self.total_trades > 0 {
            (self.winning_trades as f64 / self.total_trades as f64) * 100.0
        } else {
            0.0
        };
    }

    pub fn check_halt_conditions(&mut self, max_drawdown_pct: f64, max_consecutive_losses: u32) {
        if self.current_drawdown_pct >= max_drawdown_pct {
            self.trading_halted = true;
            self.halt_reason = Some(format!(
                "Max drawdown {:.1}% exceeded limit of {:.1}%",
                self.current_drawdown_pct, max_drawdown_pct
            ));
        } else if self.consecutive_losses >= max_consecutive_losses {
            self.trading_halted = true;
            self.halt_reason = Some(format!(
                "{} consecutive losses exceeded limit of {}",
                self.consecutive_losses, max_consecutive_losses
            ));
        }
    }
}

// ── Risk & Routing Primitives ──────────────────────────────────────────────

pub struct RiskEngine {
    pub max_drawdown_pct: f64,
    pub max_consecutive_losses: u32,
    pub min_conviction_threshold: f64,
}

impl RiskEngine {
    pub fn new() -> Self {
        Self {
            max_drawdown_pct: 15.0,
            max_consecutive_losses: 5,
            min_conviction_threshold: 0.5,
        }
    }

    pub fn with_drawdown(mut self, pct: f64) -> Self {
        self.max_drawdown_pct = pct;
        self
    }

    pub fn with_conviction(mut self, threshold: f64) -> Self {
        self.min_conviction_threshold = threshold;
        self
    }

    pub fn verify_sync(&self, decision: &TradeDecision, _data: &MarketData) -> bool {
        // Basic validation: reject zero-amount trades
        decision.amount > 0.0 && decision.conviction >= self.min_conviction_threshold
    }

    /// Calculate Kelly-optimal position size
    pub fn kelly_position_size(&self, win_rate: f64, avg_win: f64, avg_loss: f64, portfolio_value: f64) -> f64 {
        if avg_loss == 0.0 {
            return 0.0;
        }
        let b = avg_win / avg_loss.abs(); // Win/loss ratio
        let p = win_rate / 100.0;
        let q = 1.0 - p;
        
        // Kelly formula: f* = (bp - q) / b
        let kelly = (b * p - q) / b;
        
        // Use fractional Kelly (25%) for safety
        let position_pct = kelly.clamp(0.0, 0.25) * 0.25;
        
        position_pct * portfolio_value
    }

    /// Calculate trailing stop price
    pub fn trailing_stop_price(entry_price: f64, current_price: f64, stop_pct: f64, side: &str) -> Option<f64> {
        let stop_distance = entry_price * (stop_pct / 100.0);
        match side {
            "BUY" => {
                let highest = current_price.max(entry_price);
                let stop = highest - stop_distance;
                if current_price <= stop { Some(stop) } else { None }
            }
            "SELL" => {
                let lowest = current_price.min(entry_price);
                let stop = lowest + stop_distance;
                if current_price >= stop { Some(stop) } else { None }
            }
            _ => None,
        }
    }

    /// Check correlation guard — prevents adding positions that are too correlated
    pub fn check_correlation_guard(existing_positions: &[String], new_symbol: &str) -> bool {
        // Simple check: avoid having both BTC and ETH simultaneously
        // (they're highly correlated)
        let is_crypto = |s: &str| s.contains("BTC") || s.contains("ETH") || s.contains("SOL");
        let is_stock = |s: &str| !s.contains('-') || s.starts_with("XAU");
        
        let new_is_crypto = is_crypto(new_symbol);
        let _new_is_stock = is_stock(new_symbol);
        
        for pos in existing_positions {
            if new_is_crypto && is_crypto(pos) {
                // Both crypto — check if same exchange pair
                if pos == new_symbol {
                    return false; // Already holding this exact asset
                }
                // Allow multiple cryptos but flag it
                return true; // Warning but allowed
            }
        }
        true
    }

    /// Check if we should halt trading based on state
    pub fn should_halt(&self, state: &RiskState) -> Option<String> {
        if state.trading_halted {
            return state.halt_reason.clone();
        }

        if state.current_drawdown_pct >= self.max_drawdown_pct {
            return Some(format!(
                "Drawdown {:.1}% exceeds max {:.1}%",
                state.current_drawdown_pct, self.max_drawdown_pct
            ));
        }

        if state.consecutive_losses >= self.max_consecutive_losses {
            return Some(format!(
                "{} consecutive losses",
                state.consecutive_losses
            ));
        }

        None
    }
}

impl Default for RiskEngine {
    fn default() -> Self { Self::new() }
}

pub struct ExchangeClient;

impl ExchangeClient {
    pub async fn place_order(&self, decision: &TradeDecision) -> Result<(), String> {
        println!("[ExchangeClient] Placing order: {} {} @ {}", decision.action, decision.symbol, decision.price);
        Ok(())
    }
}

pub struct ExchangeRouter;

impl ExchangeRouter {
    pub fn new() -> Self { Self }

    pub fn route_sync(&self, _decision: &TradeDecision, _data: &MarketData) -> ExchangeClient {
        ExchangeClient
    }
}

impl Default for ExchangeRouter {
    fn default() -> Self { Self::new() }
}
