use crate::{
    Candle, MarketAnalysisContext, SignalDirection, SkillCategory, SkillError, SkillSignal,
    TradingSkill,
};
use std::collections::HashMap;

// ── Helper Functions ──────────────────────────────────────────────────────

fn sma(data: &[f64], period: usize) -> Vec<f64> {
    if data.len() < period || period == 0 {
        return vec![];
    }
    let mut result = Vec::with_capacity(data.len());
    for i in 0..data.len() {
        if i + 1 < period {
            result.push(f64::NAN);
        } else {
            let sum: f64 = data[i + 1 - period..=i].iter().sum();
            result.push(sum / period as f64);
        }
    }
    result
}

fn ema(data: &[f64], period: usize) -> Vec<f64> {
    if data.len() < period || period == 0 {
        return vec![];
    }
    let multiplier = 2.0 / (period as f64 + 1.0);
    let mut result = Vec::with_capacity(data.len());
    // First EMA value is SMA of first `period` elements
    let initial_sma: f64 = data[..period].iter().sum::<f64>() / period as f64;
    for i in 0..data.len() {
        if i + 1 < period {
            result.push(f64::NAN);
        } else if i + 1 == period {
            result.push(initial_sma);
        } else {
            let prev_ema = result[i - 1];
            result.push((data[i] - prev_ema) * multiplier + prev_ema);
        }
    }
    result
}

pub fn standard_deviation(data: &[f64], mean: f64) -> f64 {
    if data.len() <= 1 {
        return 0.0;
    }
    let variance = data.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / data.len() as f64;
    variance.sqrt()
}

fn candles_to_closes(candles: &[Candle]) -> Vec<f64> {
    candles.iter().map(|c| c.close).collect()
}

fn candles_to_highs(candles: &[Candle]) -> Vec<f64> {
    candles.iter().map(|c| c.high).collect()
}

fn candles_to_lows(candles: &[Candle]) -> Vec<f64> {
    candles.iter().map(|c| c.low).collect()
}

// ── 1. RSI Skill ──────────────────────────────────────────────────────────

pub struct RsiSkill {
    pub period: usize,
    pub oversold_threshold: f64,
    pub overbought_threshold: f64,
}

impl Default for RsiSkill {
    fn default() -> Self {
        Self {
            period: 14,
            oversold_threshold: 30.0,
            overbought_threshold: 70.0,
        }
    }
}

#[async_trait::async_trait]
impl TradingSkill for RsiSkill {
    fn id(&self) -> &'static str {
        "rsi"
    }
    fn name(&self) -> &'static str {
        "RSI (Relative Strength Index)"
    }
    fn description(&self) -> &'static str {
        "Measures magnitude of recent price changes to evaluate overbought/oversold conditions"
    }
    fn category(&self) -> SkillCategory {
        SkillCategory::TechnicalAnalysis
    }

    async fn analyze(&self, context: &MarketAnalysisContext) -> Result<SkillSignal, SkillError> {
        let closes = candles_to_closes(&context.candles);
        if closes.len() < self.period + 1 {
            return Err(SkillError::InsufficientData(format!(
                "Need at least {} candles for RSI, got {}",
                self.period + 1,
                closes.len()
            )));
        }

        // Calculate price changes
        let mut gains = Vec::new();
        let mut losses = Vec::new();
        for i in 1..closes.len() {
            let diff = closes[i] - closes[i - 1];
            gains.push(diff.max(0.0));
            losses.push((-diff).max(0.0));
        }

        // RSI computation using Wilder's smoothing
        let avg_gain = gains[..self.period].iter().sum::<f64>() / self.period as f64;
        let avg_loss = losses[..self.period].iter().sum::<f64>() / self.period as f64;

        let mut current_avg_gain = avg_gain;
        let mut current_avg_loss = avg_loss;

        for i in self.period..gains.len() {
            current_avg_gain =
                (current_avg_gain * (self.period as f64 - 1.0) + gains[i]) / self.period as f64;
            current_avg_loss =
                (current_avg_loss * (self.period as f64 - 1.0) + losses[i]) / self.period as f64;
        }

        let rsi = if current_avg_loss == 0.0 {
            100.0
        } else {
            100.0 - (100.0 / (1.0 + current_avg_gain / current_avg_loss))
        };

        let direction = if rsi > self.overbought_threshold {
            SignalDirection::Bearish
        } else if rsi < self.oversold_threshold {
            SignalDirection::Bullish
        } else {
            SignalDirection::Neutral
        };

        let strength = if !(20.0..=80.0).contains(&rsi) {
            0.9
        } else if rsi > self.overbought_threshold || rsi < self.oversold_threshold {
            0.6
        } else {
            0.3
        };

        let mut indicators = HashMap::new();
        indicators.insert("rsi".to_string(), rsi);
        indicators.insert("period".to_string(), self.period as f64);

        Ok(SkillSignal {
            skill_id: self.id().to_string(),
            skill_name: self.name().to_string(),
            direction,
            strength,
            confidence: 0.75,
            details: format!(
                "RSI({}) = {:.1}. {} conditions detected (threshold: {} overbought / {} oversold).",
                self.period,
                rsi,
                if rsi > self.overbought_threshold {
                    "Overbought"
                } else if rsi < self.oversold_threshold {
                    "Oversold"
                } else {
                    "Neutral range"
                },
                self.overbought_threshold,
                self.oversold_threshold
            ),
            indicators,
            time_frame: "auto".to_string(),
        })
    }
}

// ── 2. MACD Skill ─────────────────────────────────────────────────────────

pub struct MacdSkill {
    pub fast_period: usize,
    pub slow_period: usize,
    pub signal_period: usize,
}

impl Default for MacdSkill {
    fn default() -> Self {
        Self {
            fast_period: 12,
            slow_period: 26,
            signal_period: 9,
        }
    }
}

#[async_trait::async_trait]
impl TradingSkill for MacdSkill {
    fn id(&self) -> &'static str {
        "macd"
    }
    fn name(&self) -> &'static str {
        "MACD (Moving Average Convergence Divergence)"
    }
    fn description(&self) -> &'static str {
        "Trend-following momentum indicator showing relationship between two moving averages"
    }
    fn category(&self) -> SkillCategory {
        SkillCategory::TechnicalAnalysis
    }

    async fn analyze(&self, context: &MarketAnalysisContext) -> Result<SkillSignal, SkillError> {
        let closes = candles_to_closes(&context.candles);
        let min_candles = self.slow_period + self.signal_period;
        if closes.len() < min_candles {
            return Err(SkillError::InsufficientData(format!(
                "Need at least {} candles for MACD, got {}",
                min_candles,
                closes.len()
            )));
        }

        let fast_ema = ema(&closes, self.fast_period);
        let slow_ema = ema(&closes, self.slow_period);
        let mut macd_line = Vec::new();

        for i in 0..closes.len() {
            if fast_ema[i].is_nan() || slow_ema[i].is_nan() {
                macd_line.push(f64::NAN);
            } else {
                macd_line.push(fast_ema[i] - slow_ema[i]);
            }
        }

        let valid_macd: Vec<f64> = macd_line.iter().filter(|v| !v.is_nan()).copied().collect();
        let signal_line = ema(&valid_macd, self.signal_period);

        // Get the latest values
        let last_macd = *macd_line.iter().rev().find(|v| !v.is_nan()).unwrap_or(&0.0);
        let prev_macd = macd_line
            .iter()
            .rev()
            .skip(1)
            .find(|v| !v.is_nan())
            .unwrap_or(&0.0);
        let last_signal = signal_line
            .iter()
            .rev()
            .find(|v| !v.is_nan())
            .unwrap_or(&0.0);
        let histogram = last_macd - last_signal;

        // Detect crossovers
        let prev_hist = prev_macd
            - signal_line
                .iter()
                .rev()
                .skip(1)
                .find(|v| !v.is_nan())
                .unwrap_or(&0.0);

        let direction = if histogram > 0.0 && prev_hist <= 0.0 {
            SignalDirection::Bullish // Bullish crossover
        } else if histogram < 0.0 && prev_hist >= 0.0 {
            SignalDirection::Bearish // Bearish crossover
        } else if histogram > 0.0 {
            SignalDirection::Bullish
        } else {
            SignalDirection::Bearish
        };

        let strength = (histogram.abs() / context.current_price.max(0.01) * 100.0).clamp(0.1, 0.9);

        let mut indicators = HashMap::new();
        indicators.insert("macd_line".to_string(), last_macd);
        indicators.insert("signal_line".to_string(), *last_signal);
        indicators.insert("histogram".to_string(), histogram);

        Ok(SkillSignal {
            skill_id: self.id().to_string(),
            skill_name: self.name().to_string(),
            direction,
            strength,
            confidence: 0.7,
            details: format!(
                "MACD({},{},{}) histogram = {:.2}. MACD Line = {:.2}, Signal = {:.2}. {} momentum.",
                self.fast_period,
                self.slow_period,
                self.signal_period,
                histogram,
                last_macd,
                last_signal,
                if histogram > 0.0 {
                    "Bullish"
                } else {
                    "Bearish"
                }
            ),
            indicators,
            time_frame: "auto".to_string(),
        })
    }
}

// ── 3. Bollinger Bands Skill ──────────────────────────────────────────────

pub struct BollingerBandsSkill {
    pub period: usize,
    pub std_dev: f64,
}

impl Default for BollingerBandsSkill {
    fn default() -> Self {
        Self {
            period: 20,
            std_dev: 2.0,
        }
    }
}

#[async_trait::async_trait]
impl TradingSkill for BollingerBandsSkill {
    fn id(&self) -> &'static str {
        "bollinger"
    }
    fn name(&self) -> &'static str {
        "Bollinger Bands"
    }
    fn description(&self) -> &'static str {
        "Volatility bands placed above and below a moving average, indicating overbought/oversold conditions"
    }
    fn category(&self) -> SkillCategory {
        SkillCategory::TechnicalAnalysis
    }

    async fn analyze(&self, context: &MarketAnalysisContext) -> Result<SkillSignal, SkillError> {
        let closes = candles_to_closes(&context.candles);
        if closes.len() < self.period {
            return Err(SkillError::InsufficientData(format!(
                "Need at least {} candles for Bollinger Bands, got {}",
                self.period,
                closes.len()
            )));
        }

        let middle_band_values = sma(&closes, self.period);
        let price = context.current_price;

        // Get the latest valid SMA
        let last_sma = middle_band_values
            .iter()
            .rev()
            .find(|v| !v.is_nan())
            .copied()
            .unwrap_or(price);

        // Calculate standard deviation of the last `period` closes
        let recent_closes: Vec<f64> = closes.iter().rev().take(self.period).copied().collect();
        let std = standard_deviation(&recent_closes, last_sma);

        let upper_band = last_sma + self.std_dev * std;
        let lower_band = last_sma - self.std_dev * std;
        let bandwidth = ((upper_band - lower_band) / last_sma) * 100.0;

        let direction = if price >= upper_band {
            SignalDirection::Bearish // Price touching upper band = overextended
        } else if price <= lower_band || price > last_sma {
            SignalDirection::Bullish // Price touching lower band = potential bounce, or above SMA
        } else {
            SignalDirection::Bearish
        };

        // Squeeze detection (low bandwidth = potential breakout)
        let strength = if bandwidth < 5.0 {
            0.8 // Squeeze = high potential
        } else if price >= upper_band || price <= lower_band {
            0.75
        } else {
            0.4
        };

        let mut indicators = HashMap::new();
        indicators.insert("upper_band".to_string(), upper_band);
        indicators.insert("middle_band".to_string(), last_sma);
        indicators.insert("lower_band".to_string(), lower_band);
        indicators.insert("bandwidth".to_string(), bandwidth);
        indicators.insert("std_dev".to_string(), std);

        Ok(SkillSignal {
            skill_id: self.id().to_string(),
            skill_name: self.name().to_string(),
            direction,
            strength,
            confidence: 0.72,
            details: format!(
                "Bollinger({},{}) — Price ${:.2} is {:.1}% bandwidth (${:.2}–${:.2}). {}.",
                self.period,
                self.std_dev,
                price,
                bandwidth,
                lower_band,
                upper_band,
                if price >= upper_band {
                    "Touching upper band — potential reversal down"
                } else if price <= lower_band {
                    "Touching lower band — potential bounce up"
                } else if bandwidth < 5.0 {
                    "Squeeze detected — volatility breakout imminent"
                } else {
                    "Trading within normal range"
                }
            ),
            indicators,
            time_frame: "auto".to_string(),
        })
    }
}

// ── 4. SMA Skill ──────────────────────────────────────────────────────────

pub struct SmaSkill {
    pub period: usize,
    skill_id: String,
    skill_name: String,
    skill_desc: String,
}

impl SmaSkill {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            skill_id: format!("sma_{}", period),
            skill_name: format!("SMA ({})", period),
            skill_desc: format!("Simple Moving Average over {} periods", period),
        }
    }
}

#[async_trait::async_trait]
impl TradingSkill for SmaSkill {
    fn id(&self) -> &str {
        &self.skill_id
    }
    fn name(&self) -> &str {
        &self.skill_name
    }
    fn description(&self) -> &str {
        &self.skill_desc
    }
    fn category(&self) -> SkillCategory {
        SkillCategory::TechnicalAnalysis
    }

    async fn analyze(&self, context: &MarketAnalysisContext) -> Result<SkillSignal, SkillError> {
        let closes = candles_to_closes(&context.candles);
        if closes.len() < self.period {
            return Err(SkillError::InsufficientData(format!(
                "Need at least {} candles for SMA({}), got {}",
                self.period,
                self.period,
                closes.len()
            )));
        }

        let sma_values = sma(&closes, self.period);
        let last_sma = sma_values
            .iter()
            .rev()
            .find(|v| !v.is_nan())
            .copied()
            .unwrap_or(context.current_price);
        let price = context.current_price;

        let direction = if price > last_sma * 1.01 {
            SignalDirection::Bullish
        } else if price < last_sma * 0.99 {
            SignalDirection::Bearish
        } else {
            SignalDirection::Neutral
        };

        let deviation_pct = ((price - last_sma) / last_sma) * 100.0;
        let strength: f64 = (deviation_pct.abs() / 5.0).clamp(0.2, 0.9);

        let mut indicators = HashMap::new();
        indicators.insert(format!("sma_{}", self.period), last_sma);

        Ok(SkillSignal {
            skill_id: self.skill_id.clone(),
            skill_name: self.skill_name.clone(),
            direction,
            strength,
            confidence: 0.65,
            details: format!(
                "SMA({}) = ${:.2}. Price ${:.2} is {:.1}% {} the moving average.",
                self.period,
                last_sma,
                price,
                deviation_pct.abs(),
                if deviation_pct > 0.0 {
                    "above"
                } else {
                    "below"
                }
            ),
            indicators,
            time_frame: "auto".to_string(),
        })
    }
}

// ── 5. EMA Skill ──────────────────────────────────────────────────────────

pub struct EmaSkill {
    pub period: usize,
    skill_id: String,
    skill_name: String,
    skill_desc: String,
}

impl EmaSkill {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            skill_id: format!("ema_{}", period),
            skill_name: format!("EMA ({})", period),
            skill_desc: format!("Exponential Moving Average over {} periods", period),
        }
    }
}

#[async_trait::async_trait]
impl TradingSkill for EmaSkill {
    fn id(&self) -> &str {
        &self.skill_id
    }
    fn name(&self) -> &str {
        &self.skill_name
    }
    fn description(&self) -> &str {
        &self.skill_desc
    }
    fn category(&self) -> SkillCategory {
        SkillCategory::TechnicalAnalysis
    }

    async fn analyze(&self, context: &MarketAnalysisContext) -> Result<SkillSignal, SkillError> {
        let closes = candles_to_closes(&context.candles);
        if closes.len() < self.period {
            return Err(SkillError::InsufficientData(format!(
                "Need at least {} candles for EMA({}), got {}",
                self.period,
                self.period,
                closes.len()
            )));
        }

        let ema_values = ema(&closes, self.period);
        let last_ema = ema_values
            .iter()
            .rev()
            .find(|v| !v.is_nan())
            .copied()
            .unwrap_or(context.current_price);
        let price = context.current_price;

        let direction = if price > last_ema * 1.005 {
            SignalDirection::Bullish
        } else if price < last_ema * 0.995 {
            SignalDirection::Bearish
        } else {
            SignalDirection::Neutral
        };

        let deviation_pct = ((price - last_ema) / last_ema) * 100.0;
        let strength: f64 = (deviation_pct.abs() / 3.0).clamp(0.2, 0.9);

        let mut indicators = HashMap::new();
        indicators.insert(format!("ema_{}", self.period), last_ema);

        Ok(SkillSignal {
            skill_id: self.skill_id.clone(),
            skill_name: self.skill_name.clone(),
            direction,
            strength,
            confidence: 0.68,
            details: format!(
                "EMA({}) = ${:.2}. Price ${:.2} is {:.1}% {} the exponential average.",
                self.period,
                last_ema,
                price,
                deviation_pct.abs(),
                if deviation_pct > 0.0 {
                    "above"
                } else {
                    "below"
                }
            ),
            indicators,
            time_frame: "auto".to_string(),
        })
    }
}

// ── 6. Support & Resistance Skill ─────────────────────────────────────────

pub struct SupportResistanceSkill {
    pub lookback: usize,
    pub sensitivity: f64,
}

impl Default for SupportResistanceSkill {
    fn default() -> Self {
        Self {
            lookback: 40,
            sensitivity: 0.02,
        }
    }
}

#[async_trait::async_trait]
impl TradingSkill for SupportResistanceSkill {
    fn id(&self) -> &'static str {
        "support_resistance"
    }
    fn name(&self) -> &'static str {
        "Support & Resistance Levels"
    }
    fn description(&self) -> &'static str {
        "Identifies key price levels where buying or selling pressure is expected"
    }
    fn category(&self) -> SkillCategory {
        SkillCategory::TechnicalAnalysis
    }

    async fn analyze(&self, context: &MarketAnalysisContext) -> Result<SkillSignal, SkillError> {
        let highs = candles_to_highs(&context.candles);
        let lows = candles_to_lows(&context.candles);

        if highs.len() < 10 {
            return Err(SkillError::InsufficientData(
                "Not enough data for S/R analysis".to_string(),
            ));
        }

        let price = context.current_price;

        // Find local maxima (resistance) and minima (support)
        let mut resistances: Vec<f64> = Vec::new();
        let mut supports: Vec<f64> = Vec::new();

        let window = 5;
        for i in window..(highs.len() - window) {
            let is_high = (i - window..i).all(|j| highs[j] <= highs[i])
                && (i + 1..=i + window).all(|j| highs[j] <= highs[i]);
            if is_high {
                resistances.push(highs[i]);
            }

            let is_low = (i - window..i).all(|j| lows[j] >= lows[i])
                && (i + 1..=i + window).all(|j| lows[j] >= lows[i]);
            if is_low {
                supports.push(lows[i]);
            }
        }

        // Cluster nearby levels
        let cluster_threshold = price * self.sensitivity;
        let cluster = |levels: &mut Vec<f64>| {
            levels.sort_by(|a, b| a.partial_cmp(b).expect("Price levels should not be NaN"));
            let mut clustered: Vec<f64> = Vec::new();
            let mut i = 0;
            while i < levels.len() {
                let mut sum = levels[i];
                let mut count = 1;
                while i + 1 < levels.len() && (levels[i + 1] - levels[i]).abs() < cluster_threshold
                {
                    i += 1;
                    sum += levels[i];
                    count += 1;
                }
                clustered.push(sum / count as f64);
                i += 1;
            }
            clustered
        };

        let mut res_clustered = resistances;
        let mut sup_clustered = supports;

        let resistance_levels = cluster(&mut res_clustered);
        let support_levels = cluster(&mut sup_clustered);

        // Find nearest levels
        let nearest_resistance = resistance_levels
            .iter()
            .filter(|&&r| r > price)
            .min_by(|a, b| a.partial_cmp(b).expect("Price levels should not be NaN"))
            .copied();

        let nearest_support = support_levels
            .iter()
            .filter(|&&s| s < price)
            .max_by(|a, b| a.partial_cmp(b).expect("Price levels should not be NaN"))
            .copied();

        // Calculate distance to nearest levels
        let resistance_dist = nearest_resistance
            .map(|r| ((r - price) / price) * 100.0)
            .unwrap_or(5.0);
        let support_dist = nearest_support
            .map(|s| ((price - s) / price) * 100.0)
            .unwrap_or(5.0);

        // Determine direction based on proximity to levels
        let direction = if resistance_dist < 1.0 {
            SignalDirection::Bearish // Near resistance = potential rejection
        } else if support_dist < 1.0 {
            SignalDirection::Bullish // Near support = potential bounce
        } else if resistance_dist < support_dist {
            SignalDirection::Bearish // Closer to resistance
        } else {
            SignalDirection::Bullish // Closer to support
        };

        let min_dist = resistance_dist.min(support_dist);
        let strength = (1.0 - (min_dist / 10.0).min(1.0)).max(0.3);

        let mut indicators = HashMap::new();
        indicators.insert(
            "nearest_resistance".to_string(),
            nearest_resistance.unwrap_or(0.0),
        );
        indicators.insert(
            "nearest_support".to_string(),
            nearest_support.unwrap_or(0.0),
        );
        indicators.insert("resistance_distance_pct".to_string(), resistance_dist);
        indicators.insert("support_distance_pct".to_string(), support_dist);

        Ok(SkillSignal {
            skill_id: self.id().to_string(),
            skill_name: self.name().to_string(),
            direction,
            strength,
            confidence: 0.65,
            details: format!(
                "S/R Analysis: Nearest Resistance = ${:.2} ({:.1}% away), Nearest Support = ${:.2} ({:.1}% away). {}.",
                nearest_resistance.unwrap_or(0.0), resistance_dist,
                nearest_support.unwrap_or(0.0), support_dist,
                if resistance_dist < 1.0 { "Price at resistance — caution for reversal" }
                else if support_dist < 1.0 { "Price at support — potential bounce zone" }
                else { "Price in no-man's land between levels" }
            ),
            indicators,
            time_frame: "auto".to_string(),
        })
    }
}

// ── 7. Volume Analysis Skill ──────────────────────────────────────────────

pub struct VolumeAnalysisSkill;

#[async_trait::async_trait]
impl TradingSkill for VolumeAnalysisSkill {
    fn id(&self) -> &'static str {
        "volume_analysis"
    }
    fn name(&self) -> &'static str {
        "Volume Analysis"
    }
    fn description(&self) -> &'static str {
        "Analyzes trading volume to confirm price trends and detect anomalies"
    }
    fn category(&self) -> SkillCategory {
        SkillCategory::TechnicalAnalysis
    }

    async fn analyze(&self, context: &MarketAnalysisContext) -> Result<SkillSignal, SkillError> {
        let volumes: Vec<f64> = context.candles.iter().map(|c| c.volume).collect();
        if volumes.len() < 10 {
            return Err(SkillError::InsufficientData(
                "Not enough volume data".to_string(),
            ));
        }

        let recent = &volumes[volumes.len().saturating_sub(5)..];
        let avg_volume: f64 = volumes.iter().sum::<f64>() / volumes.len() as f64;
        let recent_avg: f64 = recent.iter().sum::<f64>() / recent.len() as f64;
        let volume_ratio = recent_avg / avg_volume.max(0.001);

        let price_trend = if context.candles.len() >= 5 {
            let old = context.candles[context.candles.len() - 5].close;
            let new = context.current_price;
            (new - old) / old
        } else {
            0.0
        };

        // Volume confirmation: rising volume + rising price = bullish
        let direction = if volume_ratio > 1.2 && price_trend > 0.02 {
            SignalDirection::Bullish
        } else if volume_ratio > 1.2 && price_trend < -0.02 {
            SignalDirection::Bearish
        } else if volume_ratio < 0.8 && price_trend.abs() < 0.01 {
            SignalDirection::Neutral
        } else if price_trend > 0.0 {
            SignalDirection::Bullish
        } else {
            SignalDirection::Bearish
        };

        let strength = (volume_ratio.min(3.0) / 3.0).max(0.2);

        let mut indicators = HashMap::new();
        indicators.insert("volume_ratio".to_string(), volume_ratio);
        indicators.insert("avg_volume".to_string(), avg_volume);
        indicators.insert("recent_avg_volume".to_string(), recent_avg);
        indicators.insert("price_trend_pct".to_string(), price_trend * 100.0);

        Ok(SkillSignal {
            skill_id: self.id().to_string(),
            skill_name: self.name().to_string(),
            direction,
            strength,
            confidence: 0.6,
            details: format!(
                "Volume Ratio (recent vs avg): {:.2}x. Price trend: {:.2}%. {}.",
                volume_ratio,
                price_trend * 100.0,
                if volume_ratio > 1.2 {
                    "Above-average volume confirming movement"
                } else if volume_ratio < 0.8 {
                    "Below-average volume — low conviction"
                } else {
                    "Normal volume levels"
                }
            ),
            indicators,
            time_frame: "auto".to_string(),
        })
    }
}
