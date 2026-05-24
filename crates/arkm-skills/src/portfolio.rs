use crate::{MarketAnalysisContext, SignalDirection, SkillCategory, SkillError, SkillSignal, TradingSkill};
use std::collections::HashMap;


// ── 1. Diversification Skill ──────────────────────────────────────────────

pub struct DiversificationSkill {
    pub max_single_asset_pct: f64,
    pub min_assets_for_diversification: usize,
}

impl Default for DiversificationSkill {
    fn default() -> Self {
        Self {
            max_single_asset_pct: 25.0,
            min_assets_for_diversification: 3,
        }
    }
}

#[async_trait::async_trait]
impl TradingSkill for DiversificationSkill {
    fn id(&self) -> &'static str { "diversification" }
    fn name(&self) -> &'static str { "Diversification Analysis" }
    fn description(&self) -> &'static str { "Analyzes portfolio diversification and concentration risk across assets" }
    fn category(&self) -> SkillCategory { SkillCategory::PortfolioAnalysis }

    async fn analyze(&self, context: &MarketAnalysisContext) -> Result<SkillSignal, SkillError> {
        let total_positions_value: f64 = context.open_positions.values().sum();
        let total_value = context.cash_available + total_positions_value;

        if total_value <= 0.0 {
            return Err(SkillError::InvalidParameters("Empty portfolio".to_string()));
        }

        let num_assets = context.open_positions.len();
        let cash_pct = (context.cash_available / total_value) * 100.0;
        let max_concentration = context.open_positions.values()
            .map(|v| (v / total_value) * 100.0)
            .fold(0.0f64, f64::max);

        let mut issues: Vec<String> = Vec::new();

        if num_assets < self.min_assets_for_diversification && num_assets > 0 {
            issues.push(format!(
                "Only {} assets in portfolio (minimum {} recommended for diversification)",
                num_assets, self.min_assets_for_diversification
            ));
        }

        if max_concentration > self.max_single_asset_pct {
            issues.push(format!(
                "Single asset concentration {:.1}% exceeds limit of {}%",
                max_concentration, self.max_single_asset_pct
            ));
        }

        let direction = if issues.is_empty() && num_assets >= self.min_assets_for_diversification {
            SignalDirection::Bullish  // Well diversified
        } else if !issues.is_empty() {
            SignalDirection::Bearish  // Needs attention
        } else {
            SignalDirection::Neutral
        };

        let strength = if num_assets == 0 {
            0.5
        } else {
            (1.0 - (max_concentration / 100.0)).clamp(0.2, 0.9)
        };

        let mut indicators = HashMap::new();
        indicators.insert("num_assets".to_string(), num_assets as f64);
        indicators.insert("cash_pct".to_string(), cash_pct);
        indicators.insert("max_concentration_pct".to_string(), max_concentration);
        indicators.insert("total_value".to_string(), total_value);

        let details = if num_assets == 0 {
            "Portfolio is 100% cash — no diversification concern but no market exposure.".to_string()
        } else if issues.is_empty() {
            format!(
                "Portfolio: {} assets, max concentration {:.1}%, cash {:.1}%. Well diversified ✓.",
                num_assets, max_concentration, cash_pct
            )
        } else {
            format!("⚠️ Diversification issues: {}", issues.join("; "))
        };

        Ok(SkillSignal {
            skill_id: self.id().to_string(),
            skill_name: self.name().to_string(),
            direction,
            strength,
            confidence: 0.75,
            details,
            indicators,
            time_frame: "current".to_string(),
        })
    }
}

// ── 2. Portfolio Health Skill ─────────────────────────────────────────────

pub struct PortfolioHealthSkill;

#[async_trait::async_trait]
impl TradingSkill for PortfolioHealthSkill {
    fn id(&self) -> &'static str { "portfolio_health" }
    fn name(&self) -> &'static str { "Portfolio Health Score" }
    fn description(&self) -> &'static str { "Calculates an overall health score considering returns, risk, and allocation" }
    fn category(&self) -> SkillCategory { SkillCategory::PortfolioAnalysis }

    async fn analyze(&self, context: &MarketAnalysisContext) -> Result<SkillSignal, SkillError> {
        let total_positions: f64 = context.open_positions.values().sum();
        let total_value = context.cash_available + total_positions;

        if total_value <= 0.0 {
            return Ok(SkillSignal {
                skill_id: self.id().to_string(),
                skill_name: self.name().to_string(),
                direction: SignalDirection::Neutral,
                strength: 0.5,
                confidence: 1.0,
                details: "Empty portfolio — neutral health score.".to_string(),
                indicators: {
                    let mut m = HashMap::new();
                    m.insert("health_score".to_string(), 50.0);
                    m.insert("exposure_ratio".to_string(), 0.0);
                    m
                },
                time_frame: "current".to_string(),
            });
        }

        // Calculate portfolio-level metrics
        let exposure_ratio = total_positions / total_value;
        let cash_buffer = context.cash_available / total_value;

        // Compute recent returns from candle data
        let recent_returns = if context.candles.len() >= 2 {
            let old_price = context.candles[context.candles.len() - 2].close;
            let new_price = context.current_price;
            ((new_price - old_price) / old_price.max(0.0001)) * 100.0
        } else {
            0.0
        };

        // Health score (0-100)
        let mut score: f64 = 50.0;

        // Cash buffer bonus (having 20-40% cash is healthy)
        if (0.2..=0.4).contains(&cash_buffer) {
            score += 15.0;
        } else if cash_buffer > 0.4 {
            score += 5.0;  // Too much cash = opportunity cost
        } else if cash_buffer < 0.1 {
            score -= 10.0; // Too little cash = risky
        }

        // Exposure balance
        if (0.3..=0.7).contains(&exposure_ratio) {
            score += 10.0;
        } else if exposure_ratio > 0.8 {
            score -= 15.0;
        }

        // Recent performance
        if recent_returns > 0.0 {
            score += 10.0;
        } else if recent_returns < -2.0 {
            score -= 10.0;
        }

        let clamped_score: f64 = score.clamp(0.0, 100.0);

        let direction = if clamped_score >= 70.0 {
            SignalDirection::Bullish
        } else if score >= 40.0 {
            SignalDirection::Neutral
        } else {
            SignalDirection::Bearish
        };

        let mut indicators = HashMap::new();
        indicators.insert("health_score".to_string(), score);
        indicators.insert("exposure_ratio".to_string(), exposure_ratio * 100.0);
        indicators.insert("cash_buffer".to_string(), cash_buffer * 100.0);
        indicators.insert("recent_return_pct".to_string(), recent_returns);

        Ok(SkillSignal {
            skill_id: self.id().to_string(),
            skill_name: self.name().to_string(),
            direction,
            strength: clamped_score / 100.0,
            confidence: 0.7,
            details: format!(
                "Portfolio Health Score: {:.0}/100. Exposure: {:.1}%, Cash: {:.1}%. {}.",
                clamped_score, exposure_ratio * 100.0, cash_buffer * 100.0,
                if clamped_score >= 70.0 { "Healthy portfolio structure" }
                else if clamped_score >= 40.0 { "Moderate — consider rebalancing" }
                else { "⚠️ Needs attention — high risk structure" }
            ),
            indicators,
            time_frame: "current".to_string(),
        })
    }
}

// ── 3. Correlation Risk Skill ─────────────────────────────────────────────

pub struct CorrelationRiskSkill;

#[async_trait::async_trait]
impl TradingSkill for CorrelationRiskSkill {
    fn id(&self) -> &'static str { "correlation_risk" }
    fn name(&self) -> &'static str { "Correlation Risk" }
    fn description(&self) -> &'static str { "Analyzes correlation between portfolio holdings to identify hidden concentration risk" }
    fn category(&self) -> SkillCategory { SkillCategory::PortfolioAnalysis }

    async fn analyze(&self, _context: &MarketAnalysisContext) -> Result<SkillSignal, SkillError> {
        // For now, this is a simplified analysis based on asset types
        // A full implementation would use historical price correlation matrices
        
        let num_positions = _context.open_positions.len() as f64;

        // Estimate correlation risk based on position count alone
        // More positions with fewer assets = higher correlation risk
        let estimated_correlation = if num_positions <= 1.0 {
            0.9  // Single asset = high self-correlation
        } else if num_positions <= 3.0 {
            0.6
        } else {
            0.3
        };

        let direction = if estimated_correlation > 0.7 {
            SignalDirection::Bearish  // High correlation = high risk
        } else if estimated_correlation < 0.4 {
            SignalDirection::Bullish  // Low correlation = well hedged
        } else {
            SignalDirection::Neutral
        };

        let mut indicators = HashMap::new();
        indicators.insert("estimated_correlation".to_string(), estimated_correlation);
        indicators.insert("num_assets_analyzed".to_string(), num_positions);

        Ok(SkillSignal {
            skill_id: self.id().to_string(),
            skill_name: self.name().to_string(),
            direction,
            strength: estimated_correlation,
            confidence: 0.5,
            details: format!(
                "Estimated portfolio correlation: {:.2}. {} correlation risk — {}.",
                estimated_correlation,
                if estimated_correlation > 0.7 { "HIGH" }
                else if estimated_correlation < 0.4 { "LOW" }
                else { "MODERATE" },
                if num_positions <= 1.0 {
                    "Single asset portfolio — highly concentrated"
                } else if num_positions <= 3.0 {
                    "Few assets — consider adding uncorrelated positions"
                } else {
                    "Multiple assets — reasonable diversification"
                }
            ),
            indicators,
            time_frame: "current".to_string(),
        })
    }
}
