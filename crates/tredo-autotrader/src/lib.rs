pub mod config;
pub mod loop_engine;
pub mod regime;

pub use config::AutoTradingConfig;
pub use loop_engine::{AutoTradingLoop, DecisionOutcome, TradeAction, TradingState};
pub use regime::{MarketRegime, RegimeDetector};
