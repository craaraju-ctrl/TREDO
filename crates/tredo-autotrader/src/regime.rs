use tredo_core::Candle;
use serde::{Deserialize, Serialize};

/// Market regime classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarketRegime {
    TrendingBullish,
    TrendingBearish,
    Ranging,
    HighVolatility,
    LowVolatility,
}

impl MarketRegime {
    pub fn label(&self) -> &'static str {
        match self {
            MarketRegime::TrendingBullish => "trending_bullish",
            MarketRegime::TrendingBearish => "trending_bearish",
            MarketRegime::Ranging => "ranging",
            MarketRegime::HighVolatility => "high_volatility",
            MarketRegime::LowVolatility => "low_volatility",
        }
    }

    /// Returns strategies that perform well in this regime
    pub fn recommended_strategies(&self) -> Vec<&'static str> {
        match self {
            MarketRegime::TrendingBullish => vec![
                "macd", "ema_12", "sma_20", "volume_analysis", "bollinger",
                "support_resistance",
            ],
            MarketRegime::TrendingBearish => vec![
                "macd", "ema_26", "sma_50", "volume_analysis", "support_resistance",
            ],
            MarketRegime::Ranging => vec![
                "rsi", "bollinger", "support_resistance", "volume_analysis",
            ],
            MarketRegime::HighVolatility => vec![
                "volatility_analysis", "value_at_risk", "position_sizing",
                "bollinger", "exposure_limit",
            ],
            MarketRegime::LowVolatility => vec![
                "rsi", "sma_20", "sma_50", "volume_analysis", "support_resistance",
            ],
        }
    }

    /// Returns risk weight multiplier (0.0 to 1.0)
    pub fn risk_multiplier(&self) -> f64 {
        match self {
            MarketRegime::TrendingBullish => 1.0,
            MarketRegime::TrendingBearish => 0.7,
            MarketRegime::Ranging => 0.5,
            MarketRegime::HighVolatility => 0.3,
            MarketRegime::LowVolatility => 0.8,
        }
    }
}

/// Detects the current market regime from candle data
pub struct RegimeDetector;

impl RegimeDetector {
    pub fn new() -> Self {
        Self
    }

    /// Analyze candles to determine market regime
    pub fn detect(&self, candles: &[Candle]) -> MarketRegime {
        if candles.len() < 20 {
            return MarketRegime::Ranging;
        }

        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
        let highs: Vec<f64> = candles.iter().map(|c| c.high).collect();
        let lows: Vec<f64> = candles.iter().map(|c| c.low).collect();

        // Calculate returns
        let returns: Vec<f64> = closes.windows(2)
            .map(|w| (w[1] - w[0]) / w[0].max(0.0001))
            .collect();

        // Volatility (standard deviation of returns)
        let mean_return = returns.iter().sum::<f64>() / returns.len().max(1) as f64;
        let variance = returns.iter().map(|r| (r - mean_return).powi(2)).sum::<f64>() / returns.len().max(1) as f64;
        let volatility = variance.sqrt();

        // Trend strength using linear regression slope
        let n = closes.len() as f64;
        let x_mean = (n - 1.0) / 2.0;
        let y_mean = closes.iter().sum::<f64>() / n;

        let mut numerator = 0.0;
        let mut denominator = 0.0;
        for (i, &y) in closes.iter().enumerate() {
            let x = i as f64;
            numerator += (x - x_mean) * (y - y_mean);
            denominator += (x - x_mean).powi(2);
        }

        let slope = if denominator > 0.0 { numerator / denominator } else { 0.0 };
        let slope_pct = slope / y_mean.max(0.0001);

        // ADX-like: measure of trend strength
        let mut true_ranges: Vec<f64> = Vec::new();
        for i in 0..candles.len() {
            if i == 0 { continue; }
            let tr = (highs[i] - lows[i]).max(
                (highs[i] - closes[i-1]).abs()
            ).max(
                (lows[i] - closes[i-1]).abs()
            );
            true_ranges.push(tr);
        }
        let avg_tr = true_ranges.iter().sum::<f64>() / true_ranges.len().max(1) as f64;
        let atr_pct = avg_tr / y_mean.max(0.0001);

        // Classification logic
        let is_high_vol = volatility > 0.015 || atr_pct > 0.04;
        let is_low_vol = volatility < 0.003 && atr_pct < 0.008;
        let is_trending = slope_pct.abs() > 0.0005;
        let is_bullish = slope_pct > 0.0;

        match (is_high_vol, is_low_vol, is_trending, is_bullish) {
            (true, _, _, _) => MarketRegime::HighVolatility,
            (_, true, _, _) => MarketRegime::LowVolatility,
            (_, _, true, true) => MarketRegime::TrendingBullish,
            (_, _, true, false) => MarketRegime::TrendingBearish,
            _ => MarketRegime::Ranging,
        }
    }
}

impl Default for RegimeDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candle(close: f64, high: f64, low: f64) -> Candle {
        Candle {
            time: 0,
            open: close,
            high,
            low,
            close,
            volume: 100.0,
        }
    }

    #[test]
    fn test_detect_trending_bullish() {
        let mut candles = Vec::new();
        let mut price = 100.0;
        for _ in 0..30 {
            price += 1.0;
            candles.push(make_candle(price, price + 0.5, price - 0.5));
        }
        let detector = RegimeDetector::new();
        let regime = detector.detect(&candles);
        assert_eq!(regime, MarketRegime::TrendingBullish);
    }

    #[test]
    fn test_detect_trending_bearish() {
        let mut candles = Vec::new();
        let mut price = 100.0;
        for _ in 0..30 {
            price -= 1.0;
            candles.push(make_candle(price, price + 0.5, price - 0.5));
        }
        let detector = RegimeDetector::new();
        let regime = detector.detect(&candles);
        assert_eq!(regime, MarketRegime::TrendingBearish);
    }
}
