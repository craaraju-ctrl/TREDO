pub mod loop_engine;
pub mod regime;
pub mod config;

pub use loop_engine::{AutoTradingLoop, TradingState, TradeAction, DecisionOutcome};
pub use regime::{MarketRegime, RegimeDetector};
pub use config::AutoTradingConfig;
