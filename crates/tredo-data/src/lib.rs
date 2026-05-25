pub mod yahoo;

pub use yahoo::YahooFinanceProvider;

use tredo_skills::Candle;
use chrono::{DateTime, Utc};

/// Supported time frames for market data
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TimeFrame {
    Min1,
    Min5,
    Min15,
    Min30,
    Hour1,
    Hour4,
    Day1,
}

impl TimeFrame {
    pub fn as_yahoo_interval(&self) -> &'static str {
        match self {
            TimeFrame::Min1 => "1m",
            TimeFrame::Min5 => "5m",
            TimeFrame::Min15 => "15m",
            TimeFrame::Min30 => "30m",
            TimeFrame::Hour1 => "60m",
            TimeFrame::Hour4 => "1h",  // Yahoo uses 1h for 1-hour; 4h not supported, use 1d
            TimeFrame::Day1 => "1d",
        }
    }

    pub fn as_yahoo_range(&self) -> &'static str {
        match self {
            TimeFrame::Min1 => "1d",
            TimeFrame::Min5 => "5d",
            TimeFrame::Min15 => "1mo",
            TimeFrame::Min30 => "1mo",
            TimeFrame::Hour1 => "3mo",
            TimeFrame::Hour4 => "6mo",
            TimeFrame::Day1 => "1y",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            TimeFrame::Min1 => "1m",
            TimeFrame::Min5 => "5m",
            TimeFrame::Min15 => "15m",
            TimeFrame::Min30 => "30m",
            TimeFrame::Hour1 => "1h",
            TimeFrame::Hour4 => "4h",
            TimeFrame::Day1 => "1d",
        }
    }
}

/// Market data point with additional metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MarketDataPoint {
    pub symbol: String,
    pub candles: Vec<Candle>,
    pub time_frame: TimeFrame,
    pub fetched_at: DateTime<Utc>,
}

/// Trait for market data providers
#[async_trait::async_trait]
pub trait MarketDataProvider: Send + Sync {
    /// Fetch OHLCV candles for a symbol at the given time frame
    async fn fetch_candles(&self, symbol: &str, timeframe: TimeFrame) -> Result<Vec<Candle>, String>;

    /// Fetch current price for a symbol
    async fn fetch_current_price(&self, symbol: &str) -> Result<f64, String>;

    /// Fetch multiple timeframes at once
    async fn fetch_multi_timeframe(&self, symbol: &str, timeframes: &[TimeFrame]) -> Vec<MarketDataPoint> {
        let mut results = Vec::new();
        for tf in timeframes {
            match self.fetch_candles(symbol, *tf).await {
                Ok(candles) => {
                    results.push(MarketDataPoint {
                        symbol: symbol.to_string(),
                        candles,
                        time_frame: *tf,
                        fetched_at: Utc::now(),
                    });
                }
                Err(e) => {
                    eprintln!("[MarketData] Failed to fetch {} {}: {}", symbol, tf.label(), e);
                }
            }
        }
        results
    }
}
