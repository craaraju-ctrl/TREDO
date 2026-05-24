use crate::{Candle, MarketAnalysisContext, SignalDirection, SkillCategory, SkillError, SkillSignal, TradingSkill};
use std::collections::HashMap;

// ── Shared Helper Functions ───────────────────────────────────────────────

fn ema_values(data: &[f64], period: usize) -> Vec<f64> {
    if data.len() < period || period == 0 {
        return vec![];
    }
    let multiplier = 2.0 / (period as f64 + 1.0);
    let mut result = Vec::with_capacity(data.len());
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

fn wilders_smoothing(data: &[f64], period: usize) -> Vec<f64> {
    if data.len() < period || period == 0 {
        return vec![];
    }
    let mut result = Vec::with_capacity(data.len());
    // Initial SMA
    let initial: f64 = data[..period].iter().sum::<f64>() / period as f64;
    for i in 0..data.len() {
        if i + 1 < period {
            result.push(f64::NAN);
        } else if i + 1 == period {
            result.push(initial);
        } else {
            let prev = result[i - 1];
            result.push(prev - (prev / period as f64) + data[i]);
        }
    }
    result
}

fn true_ranges(highs: &[f64], lows: &[f64], closes: &[f64]) -> Vec<f64> {
    let mut tr = Vec::with_capacity(highs.len().max(1));
    for i in 0..highs.len() {
        if i == 0 {
            tr.push(highs[i] - lows[i]);
        } else {
            let hl = highs[i] - lows[i];
            let hc = (highs[i] - closes[i - 1]).abs();
            let lc = (lows[i] - closes[i - 1]).abs();
            tr.push(hl.max(hc).max(lc));
        }
    }
    tr
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
fn candles_to_volumes(candles: &[Candle]) -> Vec<f64> {
    candles.iter().map(|c| c.volume).collect()
}

fn min_max_last_n(data: &[f64], n: usize) -> (f64, f64) {
    let len = data.len();
    if len == 0 { return (0.0, 0.0); }
    let start = len.saturating_sub(n);
    let slice = &data[start..];
    let min = slice.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = slice.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    (min, max)
}

// ══════════════════════════════════════════════════════════════════════════
//  1. ICHIMOKU CLOUD
// ══════════════════════════════════════════════════════════════════════════

pub struct IchimokuSkill {
    pub tenkan_period: usize,  // 9
    pub kijun_period: usize,   // 26
    pub senkou_b_period: usize, // 52
}

impl Default for IchimokuSkill {
    fn default() -> Self {
        Self { tenkan_period: 9, kijun_period: 26, senkou_b_period: 52 }
    }
}

#[async_trait::async_trait]
impl TradingSkill for IchimokuSkill {
    fn id(&self) -> &'static str { "ichimoku" }
    fn name(&self) -> &'static str { "Ichimoku Cloud" }
    fn description(&self) -> &'static str { "Comprehensive trend, support/resistance, and momentum indicator using multiple timeframes" }
    fn category(&self) -> SkillCategory { SkillCategory::TechnicalAnalysis }

    async fn analyze(&self, context: &MarketAnalysisContext) -> Result<SkillSignal, SkillError> {
        let highs = candles_to_highs(&context.candles);
        let lows = candles_to_lows(&context.candles);
        let closes = candles_to_closes(&context.candles);
        let min_req = self.senkou_b_period + self.kijun_period;
        if closes.len() < min_req {
            return Err(SkillError::InsufficientData(format!("Need {} candles for Ichimoku, got {}", min_req, closes.len())));
        }

        // Tenkan-sen (Conversion Line)
        let tenkan = |i: usize| -> f64 {
            let start = if i >= self.tenkan_period { i - self.tenkan_period + 1 } else { 0 };
            let h = highs[start..=i].iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let l = lows[start..=i].iter().cloned().fold(f64::INFINITY, f64::min);
            (h + l) / 2.0
        };

        // Kijun-sen (Base Line)
        let kijun = |i: usize| -> f64 {
            let start = if i >= self.kijun_period { i - self.kijun_period + 1 } else { 0 };
            let h = highs[start..=i].iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let l = lows[start..=i].iter().cloned().fold(f64::INFINITY, f64::min);
            (h + l) / 2.0
        };

        let last_idx = closes.len() - 1;
        let tenkan_val = tenkan(last_idx);
        let kijun_val = kijun(last_idx);
        let senkou_a = (tenkan_val + kijun_val) / 2.0;

        // Senkou Span B: (HH(52) + LL(52)) / 2
        let senkou_start = if last_idx >= self.senkou_b_period { last_idx - self.senkou_b_period + 1 } else { 0 };
        let hh = highs[senkou_start..=last_idx].iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let ll = lows[senkou_start..=last_idx].iter().cloned().fold(f64::INFINITY, f64::min);
        let senkou_b = (hh + ll) / 2.0;

        let price = context.current_price;
        let cloud_top = senkou_a.max(senkou_b);
        let cloud_bottom = senkou_a.min(senkou_b);
        let in_cloud = price >= cloud_bottom && price <= cloud_top;

        // TK Cross: Tenkan crossing above/below Kijun = trend signal
        let prev_tenkan = if last_idx >= 1 { tenkan(last_idx - 1) } else { tenkan_val };
        let prev_kijun = if last_idx >= 1 { kijun(last_idx - 1) } else { kijun_val };
        let tk_cross_bullish = prev_tenkan <= prev_kijun && tenkan_val > kijun_val;
        let tk_cross_bearish = prev_tenkan >= prev_kijun && tenkan_val < kijun_val;

        // Chikou Span: close plotted 26 periods back vs. close at that time
        let chikou_idx = last_idx.saturating_sub(self.kijun_period);
        let chikou_compare = closes[chikou_idx];
        let chikou_above = price > chikou_compare;

        let mut direction = SignalDirection::Neutral;
        let mut bullish_count = 0u32;
        let mut bearish_count = 0u32;

        if price > cloud_top { bullish_count += 1; }
        if price < cloud_bottom { bearish_count += 1; }
        if tenkan_val > kijun_val { bullish_count += 1; } else { bearish_count += 1; }
        if chikou_above { bullish_count += 1; } else { bearish_count += 1; }
        if tk_cross_bullish { bullish_count += 2; }
        if tk_cross_bearish { bearish_count += 2; }

        if bullish_count > bearish_count { direction = SignalDirection::Bullish; }
        else if bearish_count > bullish_count { direction = SignalDirection::Bearish; }

        let cloud_thickness = ((cloud_top - cloud_bottom) / price) * 100.0;
        let strength = if tk_cross_bullish || tk_cross_bearish {
            0.85
        } else if !in_cloud {
            0.7
        } else {
            0.4
        };

        let mut indicators = HashMap::new();
        indicators.insert("tenkan_sen".to_string(), tenkan_val);
        indicators.insert("kijun_sen".to_string(), kijun_val);
        indicators.insert("senkou_span_a".to_string(), senkou_a);
        indicators.insert("senkou_span_b".to_string(), senkou_b);
        indicators.insert("cloud_thickness_pct".to_string(), cloud_thickness);

        Ok(SkillSignal {
            skill_id: self.id().to_string(),
            skill_name: self.name().to_string(),
            direction,
            strength,
            confidence: 0.75,
            details: format!(
                "Ichimoku: Price ${:.2} {} cloud (${:.2}-${:.2}). TK={:.1}/{:.1}. Cloud thickness: {:.1}%. {}{}",
                price,
                if in_cloud { "inside" } else if price > cloud_top { "above" } else { "below" },
                cloud_bottom, cloud_top,
                tenkan_val, kijun_val, cloud_thickness,
                if tk_cross_bullish { " ★ BULLISH TK CROSS" } else if tk_cross_bearish { " ★ BEARISH TK CROSS" } else { "" },
                if !in_cloud && price > cloud_top { " — Price above cloud = bullish trend" }
                else if !in_cloud && price < cloud_bottom { " — Price below cloud = bearish trend" }
                else { " — Price in cloud = consolidation/ranging" }
            ),
            indicators,
            time_frame: "auto".to_string(),
        })
    }
}

// ══════════════════════════════════════════════════════════════════════════
//  2. ADX (Average Directional Index)
// ══════════════════════════════════════════════════════════════════════════

pub struct AdxSkill {
    pub period: usize,
}

impl Default for AdxSkill {
    fn default() -> Self { Self { period: 14 } }
}

#[async_trait::async_trait]
impl TradingSkill for AdxSkill {
    fn id(&self) -> &'static str { "adx" }
    fn name(&self) -> &'static str { "ADX (Average Directional Index)" }
    fn description(&self) -> &'static str { "Measures trend strength regardless of direction using Wilder's method" }
    fn category(&self) -> SkillCategory { SkillCategory::TechnicalAnalysis }

    async fn analyze(&self, context: &MarketAnalysisContext) -> Result<SkillSignal, SkillError> {
        let highs = candles_to_highs(&context.candles);
        let lows = candles_to_lows(&context.candles);
        let closes = candles_to_closes(&context.candles);
        if closes.len() < self.period * 2 {
            return Err(SkillError::InsufficientData(format!("Need {} candles for ADX, got {}", self.period * 2, closes.len())));
        }

        let tr = true_ranges(&highs, &lows, &closes);
        let tr_smooth = wilders_smoothing(&tr, self.period);

        // +DM and -DM
        let mut dm_plus = vec![0.0; highs.len()];
        let mut dm_minus = vec![0.0; highs.len()];
        for i in 1..highs.len() {
            let up_move = highs[i] - highs[i - 1];
            let down_move = lows[i - 1] - lows[i];
            dm_plus[i] = if up_move > down_move && up_move > 0.0 { up_move } else { 0.0 };
            dm_minus[i] = if down_move > up_move && down_move > 0.0 { down_move } else { 0.0 };
        }

        let dm_plus_smooth = wilders_smoothing(dm_plus.as_slice(), self.period);
        let dm_minus_smooth = wilders_smoothing(dm_minus.as_slice(), self.period);

        // DI+ and DI-
        let mut di_plus = vec![0.0; tr_smooth.len()];
        let mut di_minus = vec![0.0; tr_smooth.len()];
        for i in 0..tr_smooth.len() {
            if tr_smooth[i].is_nan() || tr_smooth[i] == 0.0 { continue; }
            di_plus[i] = (dm_plus_smooth[i] / tr_smooth[i]) * 100.0;
            di_minus[i] = (dm_minus_smooth[i] / tr_smooth[i]) * 100.0;
        }

        // DX and ADX
        let mut dx = vec![0.0; tr_smooth.len()];
        for i in 0..tr_smooth.len() {
            if tr_smooth[i].is_nan() { continue; }
            let sum = di_plus[i] + di_minus[i];
            if sum > 0.0 {
                dx[i] = ((di_plus[i] - di_minus[i]).abs() / sum) * 100.0;
            }
        }

        let adx_values = wilders_smoothing(&dx, self.period);
        let last_adx = adx_values.iter().rev().find(|v| !v.is_nan()).copied().unwrap_or(0.0);
        let last_di_plus = di_plus.iter().rev().find(|v| !v.is_nan()).copied().unwrap_or(0.0);
        let last_di_minus = di_minus.iter().rev().find(|v| !v.is_nan()).copied().unwrap_or(0.0);

        let trend_strong = last_adx >= 25.0;
        let di_bullish = last_di_plus > last_di_minus;

        let direction = if trend_strong && di_bullish {
            SignalDirection::Bullish
        } else if trend_strong && !di_bullish {
            SignalDirection::Bearish
        } else if last_adx >= 20.0 {
            if di_bullish { SignalDirection::Bullish } else { SignalDirection::Bearish }
        } else {
            SignalDirection::Neutral
        };

        let strength = (last_adx / 50.0).clamp(0.1, 0.95);

        let mut indicators = HashMap::new();
        indicators.insert("adx".to_string(), last_adx);
        indicators.insert("di_plus".to_string(), last_di_plus);
        indicators.insert("di_minus".to_string(), last_di_minus);

        Ok(SkillSignal {
            skill_id: self.id().to_string(),
            skill_name: self.name().to_string(),
            direction,
            strength,
            confidence: 0.75,
            details: format!(
                "ADX({}) = {:.1}. DI+ = {:.1}, DI- = {:.1}. {} trend. {}",
                self.period, last_adx, last_di_plus, last_di_minus,
                if trend_strong { "★ STRONG" } else if last_adx >= 20.0 { "Moderate" } else { "Weak" },
                if di_bullish { "DI+ > DI- = bullish bias" } else { "DI- > DI+ = bearish bias" }
            ),
            indicators,
            time_frame: "auto".to_string(),
        })
    }
}

// ══════════════════════════════════════════════════════════════════════════
//  3. SUPERTREND
// ══════════════════════════════════════════════════════════════════════════

pub struct SuperTrendSkill {
    pub period: usize,
    pub multiplier: f64,
}

impl Default for SuperTrendSkill {
    fn default() -> Self { Self { period: 10, multiplier: 3.0 } }
}

#[async_trait::async_trait]
impl TradingSkill for SuperTrendSkill {
    fn id(&self) -> &'static str { "supertrend" }
    fn name(&self) -> &'static str { "SuperTrend" }
    fn description(&self) -> &'static str { "Volatility-based trend following indicator using ATR for dynamic support/resistance" }
    fn category(&self) -> SkillCategory { SkillCategory::TechnicalAnalysis }

    async fn analyze(&self, context: &MarketAnalysisContext) -> Result<SkillSignal, SkillError> {
        let highs = candles_to_highs(&context.candles);
        let lows = candles_to_lows(&context.candles);
        let closes = candles_to_closes(&context.candles);
        if closes.len() < self.period + 5 {
            return Err(SkillError::InsufficientData(format!("Need {} candles for SuperTrend, got {}", self.period + 5, closes.len())));
        }

        let tr = true_ranges(&highs, &lows, &closes);
        let atr_values = wilders_smoothing(&tr, self.period);

        let mut supertrend = vec![0.0; closes.len()];
        let mut in_uptrend = vec![true; closes.len()]; // true = uptrend

        for i in self.period..closes.len() {
            if atr_values[i].is_nan() { continue; }
            let hl2 = (highs[i] + lows[i]) / 2.0;
            let atr = atr_values[i];
            let mut upper_band = hl2 + self.multiplier * atr;
            let mut lower_band = hl2 - self.multiplier * atr;

            if i > self.period {
                let prev_upper = hl2 + self.multiplier * atr_values[i - 1];
                let prev_lower = hl2 - self.multiplier * atr_values[i - 1];
                if closes[i - 1] <= prev_upper { upper_band = upper_band.min(prev_upper); }
                if closes[i - 1] >= prev_lower { lower_band = lower_band.max(prev_lower); }
            }

            if closes[i] > upper_band {
                in_uptrend[i] = true;
            } else if closes[i] < lower_band {
                in_uptrend[i] = false;
            } else {
                in_uptrend[i] = in_uptrend[i - 1];
            }

            supertrend[i] = if in_uptrend[i] { lower_band } else { upper_band };
        }

        let current_up = in_uptrend.iter().rev().copied().next().unwrap_or(true);
        let prev_up = in_uptrend.iter().rev().skip(1).copied().next().unwrap_or(true);
        let just_reversed = current_up != prev_up;

        let direction = if current_up { SignalDirection::Bullish } else { SignalDirection::Bearish };
        let strength = if just_reversed { 0.9 } else { 0.7 };

        let last_st = *supertrend.iter().rev().find(|&&v| v != 0.0).unwrap_or(&0.0);
        let dist_pct = if last_st > 0.0 {
            ((context.current_price - last_st) / last_st).abs() * 100.0
        } else {
            0.0
        };

        let mut indicators = HashMap::new();
        indicators.insert("supertrend".to_string(), last_st);
        indicators.insert("uptrend".to_string(), if current_up { 1.0 } else { 0.0 });
        indicators.insert("distance_pct".to_string(), dist_pct);

        Ok(SkillSignal {
            skill_id: self.id().to_string(),
            skill_name: self.name().to_string(),
            direction,
            strength,
            confidence: 0.72,
            details: format!(
                "SuperTrend({},{}) — Price is in {}trend. Reversal point: ${:.2} ({:.1}% away).{}",
                self.period, self.multiplier,
                if current_up { "UP" } else { "DOWN" },
                last_st, dist_pct,
                if just_reversed { " ★ JUST REVERSED!" } else { "" }
            ),
            indicators,
            time_frame: "auto".to_string(),
        })
    }
}

// ══════════════════════════════════════════════════════════════════════════
//  4. PARABOLIC SAR
// ══════════════════════════════════════════════════════════════════════════

pub struct ParabolicSarSkill {
    pub acceleration: f64,
    pub acceleration_max: f64,
    pub increment: f64,
}

impl Default for ParabolicSarSkill {
    fn default() -> Self {
        Self { acceleration: 0.02, acceleration_max: 0.20, increment: 0.02 }
    }
}

#[async_trait::async_trait]
impl TradingSkill for ParabolicSarSkill {
    fn id(&self) -> &'static str { "parabolic_sar" }
    fn name(&self) -> &'static str { "Parabolic SAR" }
    fn description(&self) -> &'static str { "Trend-following indicator that identifies potential reversals with accelerating stop levels" }
    fn category(&self) -> SkillCategory { SkillCategory::TechnicalAnalysis }

    async fn analyze(&self, context: &MarketAnalysisContext) -> Result<SkillSignal, SkillError> {
        let highs = candles_to_highs(&context.candles);
        let lows = candles_to_lows(&context.candles);
        let closes = candles_to_closes(&context.candles);
        if closes.len() < 15 {
            return Err(SkillError::InsufficientData(format!("Need 15+ candles for PSAR, got {}", closes.len())));
        }

        // Compute PSAR values iteratively
        let mut sar = vec![0.0; closes.len()];
        let mut af = self.acceleration;
        let mut ep = highs[..5].iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let init_low = lows[..5].iter().cloned().fold(f64::INFINITY, f64::min);
        sar[0] = init_low;
        let mut is_long = true;

        for i in 1..closes.len() {
            if is_long {
                sar[i] = sar[i - 1] + af * (ep - sar[i - 1]);
                // SAR cannot be higher than the previous two lows
                if i >= 2 {
                    sar[i] = sar[i].min(lows[i - 1]).min(lows[i - 2]);
                } else {
                    sar[i] = sar[i].min(lows[i - 1].min(lows[i]));
                }

                if lows[i] < sar[i] {
                    // Reverse to short
                    is_long = false;
                    sar[i] = ep; // new SAR = previous EP
                    af = self.acceleration;
                    ep = lows[i];
                } else {
                    if highs[i] > ep {
                        ep = highs[i];
                        af = (af + self.increment).min(self.acceleration_max);
                    }
                }
            } else {
                sar[i] = sar[i - 1] - af * (sar[i - 1] - ep);
                if i >= 2 {
                    sar[i] = sar[i].max(highs[i - 1]).max(highs[i - 2]);
                } else {
                    sar[i] = sar[i].max(highs[i - 1].max(highs[i]));
                }

                if highs[i] > sar[i] {
                    // Reverse to long
                    is_long = true;
                    sar[i] = ep;
                    af = self.acceleration;
                    ep = highs[i];
                } else {
                    if lows[i] < ep {
                        ep = lows[i];
                        af = (af + self.increment).min(self.acceleration_max);
                    }
                }
            }
        }

        let current_long = is_long;
        let current_sar = sar[closes.len() - 1];
        let prev_sar = if closes.len() >= 2 { sar[closes.len() - 2] } else { current_sar };
        let just_reversed = (current_long && prev_sar > current_sar) || (!current_long && prev_sar < current_sar);

        let direction = if current_long { SignalDirection::Bullish } else { SignalDirection::Bearish };
        let strength = if just_reversed { 0.88 } else { (closes.len() as f64 * af).min(0.7) };
        let dist_pct = ((context.current_price - current_sar) / current_sar).abs() * 100.0;

        let mut indicators = HashMap::new();
        indicators.insert("sar".to_string(), current_sar);
        indicators.insert("acceleration".to_string(), af);
        indicators.insert("trend".to_string(), if current_long { 1.0 } else { -1.0 });
        indicators.insert("distance_pct".to_string(), dist_pct);

        Ok(SkillSignal {
            skill_id: self.id().to_string(),
            skill_name: self.name().to_string(),
            direction,
            strength,
            confidence: 0.68,
            details: format!(
                "Parabolic SAR ({:.2}/{:.2}/{:.2}) — {}trend. SAR @ ${:.2} ({:.1}% away). AF={:.3}.{}",
                self.acceleration, self.increment, self.acceleration_max,
                if current_long { "UP" } else { "DOWN" },
                current_sar, dist_pct, af,
                if just_reversed { " ★ JUST REVERSED!" } else { "" }
            ),
            indicators,
            time_frame: "auto".to_string(),
        })
    }
}

// ══════════════════════════════════════════════════════════════════════════
//  5. KELTNER CHANNELS
// ══════════════════════════════════════════════════════════════════════════

pub struct KeltnerChannelsSkill {
    pub period: usize,
    pub multiplier: f64,
}

impl Default for KeltnerChannelsSkill {
    fn default() -> Self { Self { period: 20, multiplier: 2.0 } }
}

#[async_trait::async_trait]
impl TradingSkill for KeltnerChannelsSkill {
    fn id(&self) -> &'static str { "keltner" }
    fn name(&self) -> &'static str { "Keltner Channels" }
    fn description(&self) -> &'static str { "Volatility-based envelopes using EMA and ATR for trend and breakout detection" }
    fn category(&self) -> SkillCategory { SkillCategory::TechnicalAnalysis }

    async fn analyze(&self, context: &MarketAnalysisContext) -> Result<SkillSignal, SkillError> {
        let highs = candles_to_highs(&context.candles);
        let lows = candles_to_lows(&context.candles);
        let closes = candles_to_closes(&context.candles);
        if closes.len() < self.period + 5 {
            return Err(SkillError::InsufficientData(format!("Need {} candles for Keltner, got {}", self.period + 5, closes.len())));
        }

        let ema = ema_values(&closes, self.period);
        let tr = true_ranges(&highs, &lows, &closes);
        let atr_values = wilders_smoothing(&tr, self.period);

        let last_ema = ema.iter().rev().find(|v| !v.is_nan()).copied().unwrap_or(context.current_price);
        let last_atr = atr_values.iter().rev().find(|v| !v.is_nan()).copied().unwrap_or(0.0);

        let upper = last_ema + self.multiplier * last_atr;
        let lower = last_ema - self.multiplier * last_atr;
        let price = context.current_price;

        let bandwidth = ((upper - lower) / last_ema) * 100.0;

        let direction = if price >= upper {
            SignalDirection::Bearish // Overextended above
        } else if price <= lower || price > last_ema {
            SignalDirection::Bullish // Oversold below, or above mid-EMA
        } else {
            SignalDirection::Bearish
        };

        let strength = if price >= upper || price <= lower {
            0.75
        } else if bandwidth > 0.0 {
            let pct_pos = (price - lower) / (upper - lower);
            (pct_pos * 0.6 + 0.2).clamp(0.2, 0.8)
        } else {
            0.4
        };

        let mut indicators = HashMap::new();
        indicators.insert("middle_ema".to_string(), last_ema);
        indicators.insert("upper_channel".to_string(), upper);
        indicators.insert("lower_channel".to_string(), lower);
        indicators.insert("bandwidth_pct".to_string(), bandwidth);
        indicators.insert("atr".to_string(), last_atr);

        Ok(SkillSignal {
            skill_id: self.id().to_string(),
            skill_name: self.name().to_string(),
            direction,
            strength,
            confidence: 0.7,
            details: format!(
                "Keltner({},{}) — Price ${:.2} vs EMA ${:.2} (upper ${:.2}, lower ${:.2}). Bandwidth {:.1}%. {}.",
                self.period, self.multiplier, price, last_ema, upper, lower, bandwidth,
                if price >= upper { "Above upper band — potential reversal down" }
                else if price <= lower { "Below lower band — potential reversal up" }
                else if price > last_ema { "Above mid — bullish bias" }
                else { "Below mid — bearish bias" }
            ),
            indicators,
            time_frame: "auto".to_string(),
        })
    }
}

// ══════════════════════════════════════════════════════════════════════════
//  6. AROON
// ══════════════════════════════════════════════════════════════════════════

pub struct AroonSkill {
    pub period: usize,
}

impl Default for AroonSkill {
    fn default() -> Self { Self { period: 25 } }
}

#[async_trait::async_trait]
impl TradingSkill for AroonSkill {
    fn id(&self) -> &'static str { "aroon" }
    fn name(&self) -> &'static str { "Aroon Indicator" }
    fn description(&self) -> &'static str { "Identifies trend direction and strength by measuring time since recent highs/lows" }
    fn category(&self) -> SkillCategory { SkillCategory::TechnicalAnalysis }

    async fn analyze(&self, context: &MarketAnalysisContext) -> Result<SkillSignal, SkillError> {
        let highs = candles_to_highs(&context.candles);
        let lows = candles_to_lows(&context.candles);
        if highs.len() < self.period + 5 {
            return Err(SkillError::InsufficientData(format!("Need {} candles for Aroon, got {}", self.period + 5, highs.len())));
        }

        let len = highs.len();
        let start = len - self.period;
        let recent_highs = &highs[start..];
        let recent_lows = &lows[start..];

        let highest_idx = recent_highs.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).expect("High values should not be NaN"))
            .map(|(i, _)| i)
            .unwrap_or(0);
        let lowest_idx = recent_lows.iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).expect("Low values should not be NaN"))
            .map(|(i, _)| i)
            .unwrap_or(0);

        let periods_since_high = self.period - 1 - highest_idx;
        let periods_since_low = self.period - 1 - lowest_idx;

        let aroon_up = ((self.period - periods_since_high) as f64 / self.period as f64) * 100.0;
        let aroon_down = ((self.period - periods_since_low) as f64 / self.period as f64) * 100.0;
        let oscillator = aroon_up - aroon_down;

        let direction = if oscillator > 30.0 {
            SignalDirection::Bullish
        } else if oscillator < -30.0 {
            SignalDirection::Bearish
        } else if aroon_up > aroon_down {
            SignalDirection::Bullish
        } else {
            SignalDirection::Bearish
        };

        let up_strong = aroon_up > 70.0;
        let down_strong = aroon_down > 70.0;
        let strength = (oscillator.abs() / 100.0).clamp(0.2, 0.95);

        let mut indicators = HashMap::new();
        indicators.insert("aroon_up".to_string(), aroon_up);
        indicators.insert("aroon_down".to_string(), aroon_down);
        indicators.insert("aroon_oscillator".to_string(), oscillator);

        Ok(SkillSignal {
            skill_id: self.id().to_string(),
            skill_name: self.name().to_string(),
            direction,
            strength,
            confidence: 0.68,
            details: format!(
                "Aroon({}) — Up: {:.0}, Down: {:.0}, Osc: {:.0}. {}{}",
                self.period, aroon_up, aroon_down, oscillator,
                if up_strong { "★ Strong uptrend — recent highs" }
                else if down_strong { "★ Strong downtrend — recent lows" }
                else if oscillator > 0.0 { "Uptrend forming" }
                else { "Downtrend forming" },
                if up_strong && down_strong { " (ranging — new highs AND lows)" } else { "" }
            ),
            indicators,
            time_frame: "auto".to_string(),
        })
    }
}

// ══════════════════════════════════════════════════════════════════════════
//  7. PIVOT POINTS
// ══════════════════════════════════════════════════════════════════════════

pub struct PivotPointsSkill;

#[async_trait::async_trait]
impl TradingSkill for PivotPointsSkill {
    fn id(&self) -> &'static str { "pivot_points" }
    fn name(&self) -> &'static str { "Pivot Points" }
    fn description(&self) -> &'static str { "Classic pivot point levels for support/resistance based on previous period's high, low, close" }
    fn category(&self) -> SkillCategory { SkillCategory::TechnicalAnalysis }

    async fn analyze(&self, context: &MarketAnalysisContext) -> Result<SkillSignal, SkillError> {
        let highs = candles_to_highs(&context.candles);
        let lows = candles_to_lows(&context.candles);
        let closes = candles_to_closes(&context.candles);
        if closes.len() < 2 {
            return Err(SkillError::InsufficientData("Need at least 2 candles for pivot points".to_string()));
        }

        // Use the last complete candle (or previous day's) high/low/close
        let prev_high = if highs.len() >= 2 { highs[highs.len() - 2] } else { highs.iter().cloned().fold(f64::NEG_INFINITY, f64::max) };
        let prev_low = if lows.len() >= 2 { lows[lows.len() - 2] } else { lows.iter().cloned().fold(f64::INFINITY, f64::min) };
        let prev_close = if closes.len() >= 2 { closes[closes.len() - 2] } else { context.current_price };

        let pivot = (prev_high + prev_low + prev_close) / 3.0;
        let r1 = 2.0 * pivot - prev_low;
        let r2 = pivot + (prev_high - prev_low);
        let s1 = 2.0 * pivot - prev_high;
        let s2 = pivot - (prev_high - prev_low);

        let price = context.current_price;

        let direction = if price >= r1 {
            SignalDirection::Bullish // Above R1 = bullish momentum
        } else if price <= s1 {
            SignalDirection::Bearish // Below S1 = bearish momentum
        } else if price > pivot {
            SignalDirection::Bullish
        } else {
            SignalDirection::Bearish
        };

        // Strength based on proximity to key levels
        let dist_to_pivot = ((price - pivot) / pivot).abs() * 100.0;
        let strength = (dist_to_pivot / 3.0).clamp(0.2, 0.85);

        let mut indicators = HashMap::new();
        indicators.insert("pivot".to_string(), pivot);
        indicators.insert("r1".to_string(), r1);
        indicators.insert("r2".to_string(), r2);
        indicators.insert("s1".to_string(), s1);
        indicators.insert("s2".to_string(), s2);

        Ok(SkillSignal {
            skill_id: self.id().to_string(),
            skill_name: self.name().to_string(),
            direction,
            strength,
            confidence: 0.6,
            details: format!(
                "Pivot Points: P=${:.2} | R1=${:.2} R2=${:.2} | S1=${:.2} S2=${:.2}. Price ${:.2} is {} pivot. {}",
                pivot, r1, r2, s1, s2, price,
                if price >= r1 { "above R1" }
                else if price >= pivot { "above" }
                else if price <= s1 { "below S1" }
                else { "below" },
                if price >= r1 { "★ Strong bullish momentum — R1 broken" }
                else if price <= s1 { "★ Strong bearish momentum — S1 broken" }
                else if price > pivot { "Bullish — above pivot" }
                else { "Bearish — below pivot" }
            ),
            indicators,
            time_frame: "auto".to_string(),
        })
    }
}

// ══════════════════════════════════════════════════════════════════════════
//  8. CHANDELIER EXIT
// ══════════════════════════════════════════════════════════════════════════

pub struct ChandelierExitSkill {
    pub period: usize,
    pub multiplier: f64,
}

impl Default for ChandelierExitSkill {
    fn default() -> Self { Self { period: 22, multiplier: 3.0 } }
}

#[async_trait::async_trait]
impl TradingSkill for ChandelierExitSkill {
    fn id(&self) -> &'static str { "chandelier_exit" }
    fn name(&self) -> &'static str { "Chandelier Exit" }
    fn description(&self) -> &'static str { "Volatility-based trailing stop that sets a stop-loss below the recent high using ATR" }
    fn category(&self) -> SkillCategory { SkillCategory::TechnicalAnalysis }

    async fn analyze(&self, context: &MarketAnalysisContext) -> Result<SkillSignal, SkillError> {
        let highs = candles_to_highs(&context.candles);
        let lows = candles_to_lows(&context.candles);
        let closes = candles_to_closes(&context.candles);
        if closes.len() < self.period + 5 {
            return Err(SkillError::InsufficientData(format!("Need {} candles for Chandelier, got {}", self.period + 5, closes.len())));
        }

        let tr = true_ranges(&highs, &lows, &closes);
        let atr_values = wilders_smoothing(&tr, self.period);
        let last_atr = atr_values.iter().rev().find(|v| !v.is_nan()).copied().unwrap_or(0.0);

        let len = closes.len();
        let recent_high = highs[len.saturating_sub(self.period)..].iter()
            .cloned().fold(f64::NEG_INFINITY, f64::max);
        let recent_low = lows[len.saturating_sub(self.period)..].iter()
            .cloned().fold(f64::INFINITY, f64::min);

        let long_stop = recent_high - self.multiplier * last_atr;
        let short_stop = recent_low + self.multiplier * last_atr;

        let price = context.current_price;
        let distance_to_stop = ((price - long_stop) / long_stop).abs() * 100.0;

        let direction = if price > long_stop {
            SignalDirection::Bullish // Above long stop = uptrend intact
        } else if price < short_stop {
            SignalDirection::Bearish // Below short stop = downtrend
        } else {
            SignalDirection::Neutral
        };

        let strength = (1.0 - (distance_to_stop / 10.0).min(1.0)).max(0.2);

        let mut indicators = HashMap::new();
        indicators.insert("long_stop".to_string(), long_stop);
        indicators.insert("short_stop".to_string(), short_stop);
        indicators.insert("atr".to_string(), last_atr);
        indicators.insert("recent_high".to_string(), recent_high);
        indicators.insert("recent_low".to_string(), recent_low);

        Ok(SkillSignal {
            skill_id: self.id().to_string(),
            skill_name: self.name().to_string(),
            direction,
            strength,
            confidence: 0.65,
            details: format!(
                "Chandelier Exit({},{}) — Long stop @ ${:.2}, Short stop @ ${:.2}. Price ${:.2} is {:.1}% from stop. {}.",
                self.period, self.multiplier, long_stop, short_stop, price, distance_to_stop,
                if price > long_stop { "★ Uptrend intact — trail long stop up" }
                else if price < short_stop { "★ Downtrend — trail short stop down" }
                else { "Zone between stops — trend unclear" }
            ),
            indicators,
            time_frame: "auto".to_string(),
        })
    }
}

// ══════════════════════════════════════════════════════════════════════════
//  9. WILLIAMS %R
// ══════════════════════════════════════════════════════════════════════════

pub struct WilliamsRSkill {
    pub period: usize,
}

impl Default for WilliamsRSkill {
    fn default() -> Self { Self { period: 14 } }
}

#[async_trait::async_trait]
impl TradingSkill for WilliamsRSkill {
    fn id(&self) -> &'static str { "williams_r" }
    fn name(&self) -> &'static str { "Williams %R" }
    fn description(&self) -> &'static str { "Momentum oscillator measuring overbought/oversold levels relative to highest high" }
    fn category(&self) -> SkillCategory { SkillCategory::TechnicalAnalysis }

    async fn analyze(&self, context: &MarketAnalysisContext) -> Result<SkillSignal, SkillError> {
        let highs = candles_to_highs(&context.candles);
        let lows = candles_to_lows(&context.candles);
        let closes = candles_to_closes(&context.candles);
        if closes.len() < self.period + 1 {
            return Err(SkillError::InsufficientData(format!("Need {} candles for Williams %R, got {}", self.period + 1, closes.len())));
        }

        let (hh, ll) = min_max_last_n(&highs, self.period);
        let (ll2, _) = min_max_last_n(&lows, self.period);
        let low_min = ll2.min(ll);

        let price = context.current_price;
        let range = hh - low_min;
        let williams_r = if range > 0.0 {
            -((hh - price) / range) * 100.0
        } else {
            -50.0
        };

        let direction = if williams_r > -20.0 {
            SignalDirection::Bearish // Overbought
        } else if !((-80.0)..=(-50.0)).contains(&williams_r) {
            SignalDirection::Bullish // Oversold or bullish zone
        } else {
            SignalDirection::Bearish
        };

        let strength = if !((-80.0)..=(-20.0)).contains(&williams_r) {
            0.8
        } else {
            (williams_r.abs() / 100.0).clamp(0.2, 0.6)
        };

        let mut indicators = HashMap::new();
        indicators.insert("williams_r".to_string(), williams_r);
        indicators.insert("highest_high".to_string(), hh);
        indicators.insert("lowest_low".to_string(), low_min);

        Ok(SkillSignal {
            skill_id: self.id().to_string(),
            skill_name: self.name().to_string(),
            direction,
            strength,
            confidence: 0.7,
            details: format!(
                "Williams %R({}) = {:.1}. {} (thresholds: -20 overbought / -80 oversold). {}",
                self.period, williams_r,
                if williams_r > -20.0 { "★ OVERBOUGHT — potential reversal down" }
                else if williams_r < -80.0 { "★ OVERSOLD — potential reversal up" }
                else if williams_r > -50.0 { "Bearish zone" }
                else { "Bullish zone" },
                if williams_r.abs() < 10.0 { "Strong momentum" }
                else if williams_r.abs() > 90.0 { "Extreme exhaustion" }
                else { "Normal momentum range" }
            ),
            indicators,
            time_frame: "auto".to_string(),
        })
    }
}

// ══════════════════════════════════════════════════════════════════════════
//  10. ON-BALANCE VOLUME (OBV)
// ══════════════════════════════════════════════════════════════════════════

pub struct ObvSkill;

#[async_trait::async_trait]
impl TradingSkill for ObvSkill {
    fn id(&self) -> &'static str { "obv" }
    fn name(&self) -> &'static str { "On-Balance Volume (OBV)" }
    fn description(&self) -> &'static str { "Volume-based momentum indicator that relates volume flow to price movements" }
    fn category(&self) -> SkillCategory { SkillCategory::TechnicalAnalysis }

    async fn analyze(&self, context: &MarketAnalysisContext) -> Result<SkillSignal, SkillError> {
        let closes = candles_to_closes(&context.candles);
        let volumes = candles_to_volumes(&context.candles);
        if closes.len() < 10 {
            return Err(SkillError::InsufficientData("Need at least 10 candles for OBV".to_string()));
        }

        // Calculate OBV
        let mut obv = 0.0;
        let mut obv_values = Vec::with_capacity(closes.len());
        for i in 0..closes.len() {
            if i == 0 {
                obv = volumes[0];
            } else if closes[i] > closes[i - 1] {
                obv += volumes[i];
            } else if closes[i] < closes[i - 1] {
                obv -= volumes[i];
            }
            obv_values.push(obv);
        }

        // Calculate OBV trend: recent OBV vs older OBV
        let recent_obv = obv_values[obv_values.len() - 1];
        let obv_20_periods_ago = if obv_values.len() > 20 { obv_values[obv_values.len() - 21] } else { obv_values.first().copied().unwrap_or(0.0) };
        let obv_trend = (recent_obv - obv_20_periods_ago) / obv_20_periods_ago.abs().max(1.0);

        // OBV vs Price divergence
        let price_trend = (closes[closes.len() - 1] - closes[closes.len().saturating_sub(20)]) / closes[closes.len().saturating_sub(20)].max(0.001);
        let divergence = (obv_trend - price_trend).signum() != 0.0;

        let direction = if obv_trend > 0.02 && price_trend > 0.0 {
            SignalDirection::Bullish // Rising OBV confirming uptrend
        } else if obv_trend < -0.02 && price_trend < 0.0 {
            SignalDirection::Bearish // Falling OBV confirming downtrend
        } else if obv_trend > 0.0 && price_trend < 0.0 {
            SignalDirection::Bullish // Bullish divergence
        } else if obv_trend < 0.0 && price_trend > 0.0 {
            SignalDirection::Bearish // Bearish divergence
        } else {
            SignalDirection::Neutral
        };

        let strength = if divergence { 0.8 } else { (obv_trend.abs() * 10.0).clamp(0.2, 0.7) };

        let mut indicators = HashMap::new();
        indicators.insert("obv".to_string(), recent_obv);
        indicators.insert("obv_trend".to_string(), obv_trend);
        indicators.insert("price_trend".to_string(), price_trend);

        Ok(SkillSignal {
            skill_id: self.id().to_string(),
            skill_name: self.name().to_string(),
            direction,
            strength,
            confidence: 0.6,
            details: format!(
                "OBV: OBV trend={:.4}, Price trend={:.4}. {}.",
                obv_trend, price_trend,
                if obv_trend > 0.0 && price_trend > 0.0 { "Volume confirming uptrend — bullish" }
                else if obv_trend < 0.0 && price_trend < 0.0 { "Volume confirming downtrend — bearish" }
                else if obv_trend > 0.0 && price_trend < 0.0 { "★ BULLISH DIVERGENCE — price falling on rising volume" }
                else if obv_trend < 0.0 && price_trend > 0.0 { "★ BEARISH DIVERGENCE — price rising on falling volume" }
                else { "No clear OBV signal" }
            ),
            indicators,
            time_frame: "auto".to_string(),
        })
    }
}

// ══════════════════════════════════════════════════════════════════════════
//  11. CHAIKIN MONEY FLOW
// ══════════════════════════════════════════════════════════════════════════

pub struct ChaikinMoneyFlowSkill {
    pub period: usize,
}

impl Default for ChaikinMoneyFlowSkill {
    fn default() -> Self { Self { period: 20 } }
}

#[async_trait::async_trait]
impl TradingSkill for ChaikinMoneyFlowSkill {
    fn id(&self) -> &'static str { "chaikin_mf" }
    fn name(&self) -> &'static str { "Chaikin Money Flow (CMF)" }
    fn description(&self) -> &'static str { "Volume-weighted indicator measuring accumulation/distribution pressure over time" }
    fn category(&self) -> SkillCategory { SkillCategory::TechnicalAnalysis }

    async fn analyze(&self, context: &MarketAnalysisContext) -> Result<SkillSignal, SkillError> {
        let highs = candles_to_highs(&context.candles);
        let lows = candles_to_lows(&context.candles);
        let closes = candles_to_closes(&context.candles);
        let volumes = candles_to_volumes(&context.candles);
        if closes.len() < self.period + 5 {
            return Err(SkillError::InsufficientData(format!("Need {} candles for CMF, got {}", self.period + 5, closes.len())));
        }

        let len = closes.len();
        let start = len.saturating_sub(self.period);

        let mut money_flow_volume = 0.0;
        let mut total_volume = 0.0;

        for i in start..len {
            let range = highs[i] - lows[i];
            if range <= 0.0 { continue; }
            // Money Flow Multiplier: [(Close - Low) - (High - Close)] / (High - Low)
            let multiplier = ((closes[i] - lows[i]) - (highs[i] - closes[i])) / range;
            let mfv = multiplier * volumes[i];
            money_flow_volume += mfv;
            total_volume += volumes[i];
        }

        let cmf = if total_volume > 0.0 {
            money_flow_volume / total_volume
        } else {
            0.0
        };

        let direction = if cmf > 0.1 {
            SignalDirection::Bullish // Strong accumulation
        } else if cmf < -0.1 {
            SignalDirection::Bearish // Strong distribution
        } else if cmf > 0.0 {
            SignalDirection::Bullish
        } else {
            SignalDirection::Bearish
        };

        let strength = cmf.abs().clamp(0.1, 0.95);

        let mut indicators = HashMap::new();
        indicators.insert("cmf".to_string(), cmf);
        indicators.insert("money_flow_volume".to_string(), money_flow_volume);
        indicators.insert("total_volume".to_string(), total_volume);

        Ok(SkillSignal {
            skill_id: self.id().to_string(),
            skill_name: self.name().to_string(),
            direction,
            strength,
            confidence: 0.65,
            details: format!(
                "Chaikin Money Flow({}) = {:.3}. {}.",
                self.period, cmf,
                if cmf > 0.3 { "★ STRONG ACCUMULATION — buyers in control" }
                else if cmf > 0.1 { "Mild accumulation — buyers slightly ahead" }
                else if cmf < -0.3 { "★ STRONG DISTRIBUTION — sellers in control" }
                else if cmf < -0.1 { "Mild distribution — sellers slightly ahead" }
                else { "Neutral flow — balanced buying/selling" }
            ),
            indicators,
            time_frame: "auto".to_string(),
        })
    }
}

// ══════════════════════════════════════════════════════════════════════════
//  12. STOCHASTIC OSCILLATOR
// ══════════════════════════════════════════════════════════════════════════

pub struct StochasticSkill {
    pub k_period: usize,
    pub d_period: usize,
    pub oversold: f64,
    pub overbought: f64,
}

impl Default for StochasticSkill {
    fn default() -> Self {
        Self { k_period: 14, d_period: 3, oversold: 20.0, overbought: 80.0 }
    }
}

#[async_trait::async_trait]
impl TradingSkill for StochasticSkill {
    fn id(&self) -> &'static str { "stochastic" }
    fn name(&self) -> &'static str { "Stochastic Oscillator" }
    fn description(&self) -> &'static str { "Momentum oscillator comparing close to price range, identifying reversal zones" }
    fn category(&self) -> SkillCategory { SkillCategory::TechnicalAnalysis }

    async fn analyze(&self, context: &MarketAnalysisContext) -> Result<SkillSignal, SkillError> {
        let highs = candles_to_highs(&context.candles);
        let lows = candles_to_lows(&context.candles);
        let closes = candles_to_closes(&context.candles);
        if closes.len() < self.k_period + self.d_period + 5 {
            return Err(SkillError::InsufficientData(format!("Need {} candles for Stochastic, got {}",
                self.k_period + self.d_period + 5, closes.len())));
        }

        // Calculate %K values
        let mut k_values = Vec::with_capacity(closes.len());
        for i in 0..closes.len() {
            if i < self.k_period - 1 {
                k_values.push(f64::NAN);
            } else {
                let start = i - (self.k_period - 1);
                let hh = highs[start..=i].iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let ll = lows[start..=i].iter().cloned().fold(f64::INFINITY, f64::min);
                let range = hh - ll;
                let k = if range > 0.0 {
                    ((closes[i] - ll) / range) * 100.0
                } else {
                    50.0
                };
                k_values.push(k);
            }
        }

        // %D is SMA of %K
        let valid_k: Vec<f64> = k_values.iter().filter(|v| !v.is_nan()).copied().collect();
        let d_values = if valid_k.len() >= self.d_period {
            let mut d = Vec::new();
            for i in 0..valid_k.len() {
                if i < self.d_period - 1 {
                    d.push(f64::NAN);
                } else {
                    let sum: f64 = valid_k[i + 1 - self.d_period..=i].iter().sum();
                    d.push(sum / self.d_period as f64);
                }
            }
            d
        } else {
            vec![]
        };

        let k = k_values.iter().rev().find(|v| !v.is_nan()).copied().unwrap_or(50.0);
        let d = d_values.iter().rev().find(|v| !v.is_nan()).copied().unwrap_or(50.0);
        let prev_k = k_values.iter().rev().skip(1).find(|v| !v.is_nan()).copied().unwrap_or(k);
        let prev_d = d_values.iter().rev().skip(1).find(|v| !v.is_nan()).copied().unwrap_or(d);

        let k_cross_above_d = prev_k <= prev_d && k > d;
        let k_cross_below_d = prev_k >= prev_d && k < d;

        let direction = if k < self.oversold && k_cross_above_d {
            SignalDirection::Bullish // Oversold +  %K upcross = strong buy
        } else if k > self.overbought && k_cross_below_d {
            SignalDirection::Bearish // Overbought + %K downcross = strong sell
        } else if k < self.oversold {
            SignalDirection::Bullish
        } else if k > self.overbought {
            SignalDirection::Bearish
        } else if k > 50.0 {
            SignalDirection::Bullish
        } else {
            SignalDirection::Bearish
        };

        let strength = if (k_cross_above_d && k < self.oversold) || (k_cross_below_d && k > self.overbought) {
            0.9
        } else if k < self.oversold || k > self.overbought {
            0.7
        } else {
            (k / 100.0).clamp(0.2, 0.8)
        };

        let mut indicators = HashMap::new();
        indicators.insert("k".to_string(), k);
        indicators.insert("d".to_string(), d);
        indicators.insert("oversold".to_string(), self.oversold);
        indicators.insert("overbought".to_string(), self.overbought);

        Ok(SkillSignal {
            skill_id: self.id().to_string(),
            skill_name: self.name().to_string(),
            direction,
            strength,
            confidence: 0.72,
            details: format!(
                "Stochastic({},{}) — %K={:.1}, %D={:.1}. {}{}",
                self.k_period, self.d_period, k, d,
                if k < self.oversold && k_cross_above_d { "★ OVERSOLD + %K above %D — bullish reversal signal" }
                else if k > self.overbought && k_cross_below_d { "★ OVERBOUGHT + %K below %D — bearish reversal signal" }
                else if k < self.oversold { "Oversold zone — watch for %K upcross" }
                else if k > self.overbought { "Overbought zone — watch for %K downcross" }
                else if k > 50.0 { "Bullish momentum" }
                else { "Bearish momentum" },
                if k_cross_above_d { " (bullish cross)" } else if k_cross_below_d { " (bearish cross)" } else { "" }
            ),
            indicators,
            time_frame: "auto".to_string(),
        })
    }
}

// ══════════════════════════════════════════════════════════════════════════
//  13. DONCHIAN CHANNELS
// ══════════════════════════════════════════════════════════════════════════

pub struct DonchianChannelsSkill {
    pub period: usize,
}

impl Default for DonchianChannelsSkill {
    fn default() -> Self { Self { period: 20 } }
}

#[async_trait::async_trait]
impl TradingSkill for DonchianChannelsSkill {
    fn id(&self) -> &'static str { "donchian" }
    fn name(&self) -> &'static str { "Donchian Channels" }
    fn description(&self) -> &'static str { "Breakout detection using highest high and lowest low over a period (Turtle System)" }
    fn category(&self) -> SkillCategory { SkillCategory::TechnicalAnalysis }

    async fn analyze(&self, context: &MarketAnalysisContext) -> Result<SkillSignal, SkillError> {
        let highs = candles_to_highs(&context.candles);
        let lows = candles_to_lows(&context.candles);
        if highs.len() < self.period + 5 {
            return Err(SkillError::InsufficientData(format!("Need {} candles for Donchian, got {}", self.period + 5, highs.len())));
        }

        let (hh, _) = min_max_last_n(&highs, self.period);
        let (_, ll) = min_max_last_n(&lows, self.period);
        let middle = (hh + ll) / 2.0;
        let price = context.current_price;

        // Check if this is a new breakout
        let prev_hh = if highs.len() > self.period + 1 {
            let (ph, _) = min_max_last_n(&highs[..highs.len() - 1], self.period);
            ph
        } else { hh };
        let prev_ll = if lows.len() > self.period + 1 {
            let (_, pl) = min_max_last_n(&lows[..lows.len() - 1], self.period);
            pl
        } else { ll };

        let breakout_high = price > prev_hh && price >= hh;
        let breakout_low = price < prev_ll && price <= ll;

        let direction = if price >= hh {
            SignalDirection::Bullish // At or above upper channel = bullish
        } else if price <= ll {
            SignalDirection::Bearish // At or below lower channel = bearish
        } else if price > middle {
            SignalDirection::Bullish
        } else {
            SignalDirection::Bearish
        };

        let channel_width = ((hh - ll) / middle) * 100.0;
        let strength = if breakout_high || breakout_low {
            0.9
        } else if price >= hh || price <= ll {
            0.7
        } else {
            (channel_width / 10.0).clamp(0.2, 0.6)
        };

        let mut indicators = HashMap::new();
        indicators.insert("upper_channel".to_string(), hh);
        indicators.insert("lower_channel".to_string(), ll);
        indicators.insert("middle_channel".to_string(), middle);
        indicators.insert("channel_width_pct".to_string(), channel_width);

        Ok(SkillSignal {
            skill_id: self.id().to_string(),
            skill_name: self.name().to_string(),
            direction,
            strength,
            confidence: 0.7,
            details: format!(
                "Donchian({}) — Upper=${:.2}, Lower=${:.2}, Middle=${:.2}. Channel width: {:.1}%. Price ${:.2} is {}.{}{}",
                self.period, hh, ll, middle, channel_width, price,
                if price >= hh { "above upper channel" }
                else if price <= ll { "below lower channel" }
                else if price > middle { "above middle" }
                else { "below middle" },
                if breakout_high { " ★ NEW 20-PERIOD HIGH BREAKOUT! (Turtle entry signal)" }
                else if breakout_low { " ★ NEW 20-PERIOD LOW BREAKOUT!" }
                else { "" },
                if channel_width < 3.0 { " — Narrow channel = compression (breakout imminent)" }
                else { "" }
            ),
            indicators,
            time_frame: "auto".to_string(),
        })
    }
}

// ══════════════════════════════════════════════════════════════════════════
//  14. HEIKIN-ASHI
// ══════════════════════════════════════════════════════════════════════════

pub struct HeikinAshiSkill;

#[async_trait::async_trait]
impl TradingSkill for HeikinAshiSkill {
    fn id(&self) -> &'static str { "heikin_ashi" }
    fn name(&self) -> &'static str { "Heikin-Ashi Trend" }
    fn description(&self) -> &'static str { "Smoothed candlestick technique that filters market noise for clearer trend signals" }
    fn category(&self) -> SkillCategory { SkillCategory::TechnicalAnalysis }

    async fn analyze(&self, context: &MarketAnalysisContext) -> Result<SkillSignal, SkillError> {
        let highs = candles_to_highs(&context.candles);
        let lows = candles_to_lows(&context.candles);
        let opens = context.candles.iter().map(|c| c.open).collect::<Vec<_>>();
        let closes = candles_to_closes(&context.candles);
        if closes.len() < 5 {
            return Err(SkillError::InsufficientData("Need at least 5 candles for Heikin-Ashi".to_string()));
        }

        // Calculate HA candles
        let mut ha_close = Vec::with_capacity(closes.len());
        let mut ha_open = Vec::with_capacity(closes.len());
        ha_open.push((opens[0] + closes[0]) / 2.0);
        ha_close.push((opens[0] + highs[0] + lows[0] + closes[0]) / 4.0);

        let mut consecutive_up = 0u32;
        let mut consecutive_down = 0u32;

        for i in 1..closes.len() {
            let hc = (opens[i] + highs[i] + lows[i] + closes[i]) / 4.0;
            let ho = (ha_open[i - 1] + ha_close[i - 1]) / 2.0;
            ha_open.push(ho);
            ha_close.push(hc);
        }

        // Analyze last 5 HA candles for trend
        let ha_len = ha_close.len();
        let mut bullish_candles = 0u32;
        let mut bearish_candles = 0u32;

        for i in ha_len.saturating_sub(5)..ha_len {
            if ha_close[i] > ha_open[i] {
                bullish_candles += 1;
                consecutive_up += 1;
                consecutive_down = 0;
            } else {
                bearish_candles += 1;
                consecutive_down += 1;
                consecutive_up = 0;
            }
        }

        // HA color changes = trend reversals
        let just_bullish = ha_len >= 2 && ha_close[ha_len - 1] > ha_open[ha_len - 1] && ha_close[ha_len - 2] <= ha_open[ha_len - 2];
        let just_bearish = ha_len >= 2 && ha_close[ha_len - 1] <= ha_open[ha_len - 1] && ha_close[ha_len - 2] > ha_open[ha_len - 2];

        let direction = if just_bullish || bullish_candles > bearish_candles {
            SignalDirection::Bullish
        } else if just_bearish || bearish_candles > bullish_candles {
            SignalDirection::Bearish
        } else {
            SignalDirection::Neutral
        };

        let strength = if consecutive_up >= 3 || consecutive_down >= 3 {
            0.85
        } else if just_bullish || just_bearish {
            0.75
        } else {
            0.5
        };

        let ha_current = ha_close[ha_len - 1];
        let ha_prev = if ha_len >= 2 { ha_close[ha_len - 2] } else { ha_current };

        let mut indicators = HashMap::new();
        indicators.insert("ha_current".to_string(), ha_current);
        indicators.insert("ha_trend_strength".to_string(), consecutive_up.max(consecutive_down) as f64);
        indicators.insert("consecutive_up".to_string(), consecutive_up as f64);
        indicators.insert("consecutive_down".to_string(), consecutive_down as f64);

        Ok(SkillSignal {
            skill_id: self.id().to_string(),
            skill_name: self.name().to_string(),
            direction,
            strength,
            confidence: 0.72,
            details: format!(
                "Heikin-Ashi: {} consecutive {} candles. HA went from ${:.2} → ${:.2}. {}.",
                consecutive_up.max(consecutive_down),
                if consecutive_up > consecutive_down { "GREEN (bullish)" } else { "RED (bearish)" },
                ha_prev, ha_current,
                if just_bullish { "★ BULLISH REVERSAL — red to green" }
                else if just_bearish { "★ BEARISH REVERSAL — green to red" }
                else if consecutive_up >= 3 { "★ Strong uptrend — multiple green HA candles" }
                else if consecutive_down >= 3 { "★ Strong downtrend — multiple red HA candles" }
                else { "Mixed HA candles — ranging" }
            ),
            indicators,
            time_frame: "auto".to_string(),
        })
    }
}

// ══════════════════════════════════════════════════════════════════════════
//  15. MARKET STRUCTURE (Break of Structure / Order Flow)
// ══════════════════════════════════════════════════════════════════════════

pub struct MarketStructureSkill {
    pub swing_lookback: usize,
}

impl Default for MarketStructureSkill {
    fn default() -> Self { Self { swing_lookback: 5 } }
}

#[async_trait::async_trait]
impl TradingSkill for MarketStructureSkill {
    fn id(&self) -> &'static str { "market_structure" }
    fn name(&self) -> &'static str { "Market Structure (BOS/CHoCH)" }
    fn description(&self) -> &'static str { "Identifies break of structure (BOS) and change of character (CHoCH) for order-flow analysis" }
    fn category(&self) -> SkillCategory { SkillCategory::TechnicalAnalysis }

    async fn analyze(&self, context: &MarketAnalysisContext) -> Result<SkillSignal, SkillError> {
        let highs = candles_to_highs(&context.candles);
        let lows = candles_to_lows(&context.candles);
        if highs.len() < self.swing_lookback * 4 {
            return Err(SkillError::InsufficientData(format!("Need {} candles for market structure, got {}",
                self.swing_lookback * 4, highs.len())));
        }

        let len = highs.len();
        let lookback = self.swing_lookback;

        // Find swing highs and lows
        let mut swing_highs: Vec<(usize, f64)> = Vec::new();
        let mut swing_lows: Vec<(usize, f64)> = Vec::new();

        for i in lookback..(len - lookback) {
            let is_swing_high = (i - lookback..i).all(|j| highs[j] <= highs[i])
                && (i + 1..=i + lookback).all(|j| highs[j] <= highs[i]);
            if is_swing_high { swing_highs.push((i, highs[i])); }

            let is_swing_low = (i - lookback..i).all(|j| lows[j] >= lows[i])
                && (i + 1..=i + lookback).all(|j| lows[j] >= lows[i]);
            if is_swing_low { swing_lows.push((i, lows[i])); }
        }

        if swing_highs.len() < 2 || swing_lows.len() < 2 {
            return Ok(SkillSignal {
                skill_id: self.id().to_string(),
                skill_name: self.name().to_string(),
                direction: SignalDirection::Neutral,
                strength: 0.3,
                confidence: 0.5,
                details: "Not enough swing points to determine market structure. Need more data.".to_string(),
                indicators: {
                    let mut m = HashMap::new();
                    m.insert("swing_highs_count".to_string(), swing_highs.len() as f64);
                    m.insert("swing_lows_count".to_string(), swing_lows.len() as f64);
                    m
                },
                time_frame: "auto".to_string(),
            });
        }

        let latest_high = swing_highs.last().map(|(_, p)| *p).unwrap_or(0.0);
        let prev_high = if swing_highs.len() >= 2 { swing_highs[swing_highs.len() - 2].1 } else { latest_high };
        let latest_low = swing_lows.last().map(|(_, p)| *p).unwrap_or(0.0);
        let prev_low = if swing_lows.len() >= 2 { swing_lows[swing_lows.len() - 2].1 } else { latest_low };

        let price = context.current_price;
        let bos_bullish = price > prev_high; // Break of structure to the upside
        let bos_bearish = price < prev_low;  // Break of structure to the downside

        // Higher highs + higher lows = uptrend
        let higher_highs = if swing_highs.len() >= 2 { latest_high > prev_high } else { false };
        let higher_lows = if swing_lows.len() >= 2 { latest_low > prev_low } else { false };

        // Lower highs + lower lows = downtrend
        let lower_highs = if swing_highs.len() >= 2 { latest_high < prev_high } else { false };
        let lower_lows = if swing_lows.len() >= 2 { latest_low < prev_low } else { false };

        let direction = if higher_highs && higher_lows {
            SignalDirection::Bullish // Strong uptrend
        } else if lower_highs && lower_lows {
            SignalDirection::Bearish // Strong downtrend
        } else if bos_bullish {
            SignalDirection::Bullish
        } else if bos_bearish {
            SignalDirection::Bearish
        } else if higher_highs {
            SignalDirection::Bullish
        } else {
            SignalDirection::Bearish
        };

        let strength = if (higher_highs && higher_lows) || (lower_highs && lower_lows) {
            0.85
        } else if bos_bullish || bos_bearish {
            0.75
        } else {
            0.4
        };

        let mut indicators = HashMap::new();
        indicators.insert("last_swing_high".to_string(), latest_high);
        indicators.insert("last_swing_low".to_string(), latest_low);
        indicators.insert("prev_swing_high".to_string(), prev_high);
        indicators.insert("prev_swing_low".to_string(), prev_low);
        indicators.insert("bos_bullish".to_string(), if bos_bullish { 1.0 } else { 0.0 });
        indicators.insert("bos_bearish".to_string(), if bos_bearish { 1.0 } else { 0.0 });

        Ok(SkillSignal {
            skill_id: self.id().to_string(),
            skill_name: self.name().to_string(),
            direction,
            strength,
            confidence: 0.7,
            details: format!(
                "Market Structure: Price ${:.2}. {} {} {} {}.",
                price,
                if higher_highs { "HH" } else if lower_highs { "LH" } else { "—" },
                if higher_lows { "HL" } else if lower_lows { "LL" } else { "—" },
                if bos_bullish { "★ BOS UP (bullish breakout)" } else if bos_bearish { "★ BOS DOWN (bearish breakout)" } else { "No BOS" },
                if higher_highs && higher_lows { "★ UPTREND — higher highs + higher lows" }
                else if lower_highs && lower_lows { "★ DOWNTREND — lower highs + lower lows" }
                else if higher_highs { "Potential uptrend forming (higher highs)" }
                else if lower_lows { "Potential downtrend forming (lower lows)" }
                else { "Ranging — no clear structure" }
            ),
            indicators,
            time_frame: "auto".to_string(),
        })
    }
}

// ══════════════════════════════════════════════════════════════════════════
//  16. CYCLICAL / REGIME-BASED MOMENTUM (MarketCypher)
// ══════════════════════════════════════════════════════════════════════════

pub struct MarketCypherSkill {
    pub short_period: usize,
    pub long_period: usize,
}

impl Default for MarketCypherSkill {
    fn default() -> Self { Self { short_period: 5, long_period: 40 } }
}

#[async_trait::async_trait]
impl TradingSkill for MarketCypherSkill {
    fn id(&self) -> &'static str { "market_cypher" }
    fn name(&self) -> &'static str { "Market Cypher (Momentum Regime)" }
    fn description(&self) -> &'static str { "Multi-timeframe momentum analysis comparing short vs long-term velocity for regime detection" }
    fn category(&self) -> SkillCategory { SkillCategory::TechnicalAnalysis }

    async fn analyze(&self, context: &MarketAnalysisContext) -> Result<SkillSignal, SkillError> {
        let closes = candles_to_closes(&context.candles);
        if closes.len() < self.long_period + 5 {
            return Err(SkillError::InsufficientData(format!("Need {} candles for Market Cypher, got {}",
                self.long_period + 5, closes.len())));
        }

        let len = closes.len();
        let short_ret = (closes[len - 1] - closes[len - 1 - self.short_period]) / closes[len - 1 - self.short_period].max(0.001);
        let long_ret = (closes[len - 1] - closes[len - 1 - self.long_period]) / closes[len - 1 - self.long_period].max(0.001);

        // Momentum divergence: short-term vs long-term
        let momentum_ratio = if long_ret != 0.0 { short_ret / long_ret } else { short_ret.signum() };

        // Acceleration: comparing recent short-term returns to previous short-term returns
        let prev_short_ret = if len > self.short_period + 1 {
            (closes[len - 1 - self.short_period] - closes[len - 1 - 2 * self.short_period].max(closes[0]))
                / closes[len - 1 - 2 * self.short_period].max(closes[0]).max(0.001)
        } else { short_ret };
        let acceleration = short_ret - prev_short_ret;

        let direction = if short_ret > 0.0 && long_ret > 0.0 && acceleration > 0.0 {
            SignalDirection::Bullish // Strongly accelerating uptrend
        } else if short_ret < 0.0 && long_ret < 0.0 && acceleration < 0.0 {
            SignalDirection::Bearish // Strongly accelerating downtrend
        } else if short_ret > 0.0 && momentum_ratio > 1.5 {
            SignalDirection::Bullish // Short-term outpacing long-term = strong momentum
        } else if short_ret < 0.0 && momentum_ratio < -1.5 {
            SignalDirection::Bearish // Short-term declining faster = bearish momentum
        } else if short_ret > 0.0 {
            SignalDirection::Bullish
        } else if short_ret < 0.0 {
            SignalDirection::Bearish
        } else {
            SignalDirection::Neutral
        };

        let strength = (short_ret.abs() * 10.0 + acceleration.abs() * 5.0).clamp(0.15, 0.95);

        let mut indicators = HashMap::new();
        indicators.insert("short_momentum".to_string(), short_ret * 100.0);
        indicators.insert("long_momentum".to_string(), long_ret * 100.0);
        indicators.insert("acceleration".to_string(), acceleration * 100.0);
        indicators.insert("momentum_ratio".to_string(), momentum_ratio);

        Ok(SkillSignal {
            skill_id: self.id().to_string(),
            skill_name: self.name().to_string(),
            direction,
            strength,
            confidence: 0.65,
            details: format!(
                "Market Cypher: Short-term({})={:.2}%, Long-term({})={:.2}%, Accel={:.2}%. {}",
                self.short_period, short_ret * 100.0,
                self.long_period, long_ret * 100.0,
                acceleration * 100.0,
                if short_ret > 0.0 && long_ret > 0.0 && acceleration > 0.0 { "★ ALL SYSTEMS GO — accelerating uptrend" }
                else if short_ret < 0.0 && long_ret < 0.0 && acceleration < 0.0 { "★ BEARISH ACCELERATION — cascading downtrend" }
                else if short_ret > 0.0 && long_ret < 0.0 { "Short-term recovery in long-term downtrend — cautious bullish" }
                else if short_ret < 0.0 && long_ret > 0.0 { "Short-term pullback in long-term uptrend — dip buy zone?" }
                else if short_ret > 0.0 { "Short-term bullish" }
                else if short_ret < 0.0 { "Short-term bearish" }
                else { "Flat momentum" }
            ),
            indicators,
            time_frame: "auto".to_string(),
        })
    }
}
