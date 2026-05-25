use serde::{Deserialize, Serialize};

/// Configuration for the autonomous trading loop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoTradingConfig {
    /// Symbols to trade
    pub symbols: Vec<String>,

    /// Whether auto-trading is enabled
    pub enabled: bool,

    /// Whether to use paper trading (fake money) or real money
    pub paper_trading: bool,

    /// Interval in seconds between analysis cycles
    pub analysis_interval_secs: u64,

    /// Minimum conviction threshold to execute a trade (0.0 to 1.0)
    pub min_conviction: f64,

    /// Minimum number of signals required to act
    pub min_signals_required: u32,

    /// Maximum positions to hold simultaneously
    pub max_positions: usize,

    /// Maximum loss per trade as percentage of portfolio
    pub max_risk_per_trade_pct: f64,

    /// Maximum total portfolio drawdown before stopping all trading
    pub max_drawdown_pct: f64,

    /// Enable trailing stop loss
    pub trailing_stop_enabled: bool,

    /// Trailing stop percentage
    pub trailing_stop_pct: f64,

    /// Credit allocation for paper trading
    pub paper_balance: f64,
}

impl Default for AutoTradingConfig {
    fn default() -> Self {
        Self {
            symbols: vec![
                "BTC-USD".to_string(),
                "ETH-USD".to_string(),
                "SOL-USD".to_string(),
            ],
            enabled: false,
            paper_trading: true,
            analysis_interval_secs: 300, // 5 minutes
            min_conviction: 0.55,
            min_signals_required: 3,
            max_positions: 5,
            max_risk_per_trade_pct: 2.0,
            max_drawdown_pct: 15.0,
            trailing_stop_enabled: true,
            trailing_stop_pct: 3.0,
            paper_balance: 100_000.0,
        }
    }
}
