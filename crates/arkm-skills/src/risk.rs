use crate::{MarketAnalysisContext, SignalDirection, SkillCategory, SkillError, SkillSignal, TradingSkill};
use std::collections::HashMap;

// ── 1. Position Sizing Skill ──────────────────────────────────────────────

pub struct PositionSizingSkill {
    pub max_risk_per_trade_pct: f64,
    pub base_position_pct: f64,
}

impl Default for PositionSizingSkill {
    fn default() -> Self {
        Self {
            max_risk_per_trade_pct: 2.0,   // Max 2% risk per trade
            base_position_pct: 10.0,        // Base position = 10% of available cash
        }
    }
}

#[async_trait::async_trait]
impl TradingSkill for PositionSizingSkill {
    fn id(&self) -> &'static str { "position_sizing" }
    fn name(&self) -> &'static str { "Position Sizing" }
    fn description(&self) -> &'static str { "Calculates optimal position size based on risk tolerance and account equity" }
    fn category(&self) -> SkillCategory { SkillCategory::RiskAssessment }

    async fn analyze(&self, context: &MarketAnalysisContext) -> Result<SkillSignal, SkillError> {
        if context.cash_available <= 0.0 {
            return Err(SkillError::InvalidParameters("No cash available for position sizing".to_string()));
        }

        // Base position size (percentage of available cash)
        let base_position_value = context.cash_available * (self.base_position_pct / 100.0);
        
        // Risk-adjusted position (percentage of total portfolio)
        let risk_position_value = context.portfolio_value * (self.max_risk_per_trade_pct / 100.0);
        
        // Use the more conservative of the two
        let recommended_position = base_position_value.min(risk_position_value);
        let position_pct_of_cash = (recommended_position / context.cash_available) * 100.0;
        let position_pct_of_portfolio = (recommended_position / context.portfolio_value.max(0.01)) * 100.0;

        // Check if existing exposure is too high
        let total_exposure: f64 = context.open_positions.values().sum::<f64>() + context.exposure;
        let exposure_ratio = total_exposure / context.portfolio_value.max(0.01);

        let direction = if exposure_ratio > 0.5 {
            SignalDirection::Bearish  // Too much exposure, shouldn't add more
        } else if position_pct_of_cash > 0.0 {
            SignalDirection::Bullish  // Room for a position
        } else {
            SignalDirection::Neutral
        };

        let strength = (1.0 - exposure_ratio).clamp(0.1, 0.9);

        let mut indicators = HashMap::new();
        indicators.insert("recommended_position_usd".to_string(), recommended_position);
        indicators.insert("position_pct_of_cash".to_string(), position_pct_of_cash);
        indicators.insert("position_pct_of_portfolio".to_string(), position_pct_of_portfolio);
        indicators.insert("current_exposure_pct".to_string(), exposure_ratio * 100.0);
        indicators.insert("max_risk_per_trade".to_string(), self.max_risk_per_trade_pct);

        Ok(SkillSignal {
            skill_id: self.id().to_string(),
            skill_name: self.name().to_string(),
            direction,
            strength,
            confidence: 0.8,
            details: format!(
                "Position Sizing: Max position = ${:.2} ({:.1}% of cash). Current exposure: {:.1}% of portfolio. {}.",
                recommended_position, position_pct_of_cash, exposure_ratio * 100.0,
                if exposure_ratio > 0.5 { "HIGH EXPOSURE — reduce position size" }
                else { "Healthy room for new positions" }
            ),
            indicators,
            time_frame: "current".to_string(),
        })
    }
}

// ── 2. Value at Risk (VaR) Skill ──────────────────────────────────────────

pub struct ValueAtRiskSkill {
    pub confidence_level: f64,
    pub lookback_periods: usize,
}

impl Default for ValueAtRiskSkill {
    fn default() -> Self {
        Self {
            confidence_level: 0.95,
            lookback_periods: 20,
        }
    }
}

#[async_trait::async_trait]
impl TradingSkill for ValueAtRiskSkill {
    fn id(&self) -> &'static str { "value_at_risk" }
    fn name(&self) -> &'static str { "Value at Risk (VaR)" }
    fn description(&self) -> &'static str { "Estimates potential loss amount over a given time period at a given confidence level" }
    fn category(&self) -> SkillCategory { SkillCategory::RiskAssessment }

    async fn analyze(&self, context: &MarketAnalysisContext) -> Result<SkillSignal, SkillError> {
        let closes: Vec<f64> = context.candles.iter().map(|c| c.close).collect();
        if closes.len() < self.lookback_periods + 1 {
            return Err(SkillError::InsufficientData(format!(
                "Need at least {} candles for VaR, got {}", self.lookback_periods + 1, closes.len()
            )));
        }

        // Calculate daily returns
        let mut returns: Vec<f64> = Vec::new();
        for i in 1..closes.len() {
            let ret = (closes[i] - closes[i - 1]) / closes[i - 1].max(0.0001);
            returns.push(ret);
        }

        if returns.is_empty() {
            return Err(SkillError::InsufficientData("No returns calculated for VaR".to_string()));
        }

        // Sort returns to find percentile
        returns.sort_by(|a, b| a.partial_cmp(b).expect("Return values should not be NaN"));
        
        // Get the VaR at the specified confidence level
        let var_index = ((1.0 - self.confidence_level) * returns.len() as f64) as usize;
        let var_index = var_index.min(returns.len().saturating_sub(1));
        let daily_var = returns[var_index].abs();

        // Scale to position size
        let position_value = context.portfolio_value;
        let var_amount = position_value * daily_var;

        // Severity check
        let direction = if daily_var > 0.05 {
            SignalDirection::Bearish  // High VaR = risky
        } else if daily_var > 0.02 {
            SignalDirection::Neutral  // Moderate risk
        } else {
            SignalDirection::Bullish  // Low risk
        };

        let strength = (daily_var / 0.10).clamp(0.2, 0.9);

        let mut indicators = HashMap::new();
        indicators.insert("daily_var_pct".to_string(), daily_var * 100.0);
        indicators.insert("daily_var_amount".to_string(), var_amount);
        indicators.insert("confidence_level".to_string(), self.confidence_level);
        indicators.insert("portfolio_value".to_string(), position_value);

        Ok(SkillSignal {
            skill_id: self.id().to_string(),
            skill_name: self.name().to_string(),
            direction,
            strength,
            confidence: 0.7,
            details: format!(
                "VaR({:.0}%): Max expected daily loss = {:.2}% (${:.2}). {} risk profile.",
                self.confidence_level * 100.0, daily_var * 100.0, var_amount,
                if daily_var > 0.05 { "HIGH — consider reducing exposure" }
                else if daily_var > 0.02 { "MODERATE — standard trading risk" }
                else { "LOW — favorable risk environment" }
            ),
            indicators,
            time_frame: "daily".to_string(),
        })
    }
}

// ── 3. Exposure Limit Skill ───────────────────────────────────────────────

pub struct ExposureLimitSkill {
    pub max_single_exposure_pct: f64,
    pub max_total_exposure_pct: f64,
}

impl Default for ExposureLimitSkill {
    fn default() -> Self {
        Self {
            max_single_exposure_pct: 15.0,    // Max 15% in one asset
            max_total_exposure_pct: 60.0,     // Max 60% total portfolio exposure
        }
    }
}

#[async_trait::async_trait]
impl TradingSkill for ExposureLimitSkill {
    fn id(&self) -> &'static str { "exposure_limit" }
    fn name(&self) -> &'static str { "Exposure Limit Check" }
    fn description(&self) -> &'static str { "Ensures position exposure stays within defined risk limits" }
    fn category(&self) -> SkillCategory { SkillCategory::RiskAssessment }

    async fn analyze(&self, context: &MarketAnalysisContext) -> Result<SkillSignal, SkillError> {
        let single_exposure_pct = (context.exposure / context.portfolio_value.max(0.01)) * 100.0;
        let total_exposure: f64 = context.open_positions.values().sum::<f64>() + context.exposure;
        let total_exposure_pct = (total_exposure / context.portfolio_value.max(0.01)) * 100.0;

        let mut issues = Vec::new();
        if single_exposure_pct > self.max_single_exposure_pct {
            issues.push(format!("Single asset exposure {:.1}% exceeds limit of {}%", single_exposure_pct, self.max_single_exposure_pct));
        }
        if total_exposure_pct > self.max_total_exposure_pct {
            issues.push(format!("Total exposure {:.1}% exceeds limit of {}%", total_exposure_pct, self.max_total_exposure_pct));
        }

        let direction = if !issues.is_empty() {
            SignalDirection::Bearish
        } else if total_exposure_pct < self.max_total_exposure_pct * 0.5 {
            SignalDirection::Bullish
        } else {
            SignalDirection::Neutral
        };

        let strength = (total_exposure_pct / self.max_total_exposure_pct).clamp(0.2, 0.9);

        let mut indicators = HashMap::new();
        indicators.insert("single_exposure_pct".to_string(), single_exposure_pct);
        indicators.insert("total_exposure_pct".to_string(), total_exposure_pct);
        indicators.insert("max_single_limit".to_string(), self.max_single_exposure_pct);
        indicators.insert("max_total_limit".to_string(), self.max_total_exposure_pct);

        let details = if issues.is_empty() {
            format!(
                "Exposure Limits: Single asset {:.1}% (limit {}%), Total {:.1}% (limit {}%). All limits respected.",
                single_exposure_pct, self.max_single_exposure_pct,
                total_exposure_pct, self.max_total_exposure_pct
            )
        } else {
            format!("⚠️ Exposure Limit Breach: {}", issues.join("; "))
        };

        Ok(SkillSignal {
            skill_id: self.id().to_string(),
            skill_name: self.name().to_string(),
            direction,
            strength,
            confidence: 0.9,
            details,
            indicators,
            time_frame: "current".to_string(),
        })
    }
}

// ── 4. Volatility Analysis Skill ──────────────────────────────────────────

pub struct VolatilityAnalysisSkill {
    pub lookback_periods: usize,
}

impl Default for VolatilityAnalysisSkill {
    fn default() -> Self {
        Self { lookback_periods: 20 }
    }
}

#[async_trait::async_trait]
impl TradingSkill for VolatilityAnalysisSkill {
    fn id(&self) -> &'static str { "volatility_analysis" }
    fn name(&self) -> &'static str { "Volatility Analysis" }
    fn description(&self) -> &'static str { "Analyzes price volatility to identify market conditions and adjust strategy" }
    fn category(&self) -> SkillCategory { SkillCategory::RiskAssessment }

    async fn analyze(&self, context: &MarketAnalysisContext) -> Result<SkillSignal, SkillError> {
        let closes: Vec<f64> = context.candles.iter().map(|c| c.close).collect();
        if closes.len() < self.lookback_periods {
            return Err(SkillError::InsufficientData(format!(
                "Need at least {} candles for volatility analysis, got {}", self.lookback_periods, closes.len()
            )));
        }

        let recent = &closes[closes.len().saturating_sub(self.lookback_periods)..];
        let mean: f64 = recent.iter().sum::<f64>() / recent.len() as f64;
        let std = crate::technical::standard_deviation(recent, mean);
        let volatility_pct = (std / mean.max(0.0001)) * 100.0;

        // ATR-like measure using high-low
        let true_ranges: Vec<f64> = context.candles
            .iter()
            .skip(context.candles.len().saturating_sub(self.lookback_periods))
            .map(|c| c.high - c.low)
            .collect();
        let avg_range: f64 = true_ranges.iter().sum::<f64>() / true_ranges.len().max(1) as f64;
        let atr_pct = (avg_range / mean.max(0.0001)) * 100.0;

        let direction = if volatility_pct > 5.0 {
            SignalDirection::Neutral  // High vol = choppy, unpredictable
        } else if volatility_pct < 1.5 {
            SignalDirection::Bullish  // Low vol = trending opportunity
        } else {
            SignalDirection::Neutral
        };

        let strength = (volatility_pct / 10.0).clamp(0.2, 0.9);

        let mut indicators = HashMap::new();
        indicators.insert("volatility_pct".to_string(), volatility_pct);
        indicators.insert("atr".to_string(), avg_range);
        indicators.insert("atr_pct".to_string(), atr_pct);
        indicators.insert("std_dev".to_string(), std);

        Ok(SkillSignal {
            skill_id: self.id().to_string(),
            skill_name: self.name().to_string(),
            direction,
            strength,
            confidence: 0.7,
            details: format!(
                "Volatility: {:.2}% (std dev). ATR: {:.2}% of price. {}.",
                volatility_pct, atr_pct,
                if volatility_pct > 5.0 { "HIGH VOLATILITY — widen stops, reduce size" }
                else if volatility_pct < 1.5 { "LOW VOLATILITY — tight ranges, trend trades favorable" }
                else { "MODERATE VOLATILITY — standard market conditions" }
            ),
            indicators,
            time_frame: "auto".to_string(),
        })
    }
}
