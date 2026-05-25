use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use tredo_skills::{MarketAnalysisContext, SignalDirection, SkillSignal};

// ── Performance Record ────────────────────────────────────────────────────

/// Record of a single trade outcome for a specific skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillTradeRecord {
    pub id: String,
    pub symbol: String,
    pub skill_id: String,
    pub skill_name: String,
    pub direction: String,
    pub strength: f64,
    pub confidence: f64,
    pub entry_price: f64,
    pub exit_price: Option<f64>,
    pub pnl: Option<f64>,
    pub pnl_pct: Option<f64>,
    pub regime: String,
    pub timestamp: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub was_correct: Option<bool>,
}

/// Aggregate performance metrics for a single skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillPerformance {
    pub skill_id: String,
    pub skill_name: String,
    pub total_trades: u64,
    pub winning_trades: u64,
    pub losing_trades: u64,
    pub win_rate: f64,
    pub avg_win_pct: f64,
    pub avg_loss_pct: f64,
    pub profit_factor: f64,
    pub current_streak: i32, // positive = wins, negative = losses
    pub total_pnl: f64,
    pub adjusted_weight: f64,
    pub base_weight: f64,
    pub last_updated: DateTime<Utc>,
    /// Per-regime breakdown
    pub regime_performance: HashMap<String, RegimePerformance>,
}

/// Performance breakdown by market regime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimePerformance {
    pub total_trades: u64,
    pub winning_trades: u64,
    pub win_rate: f64,
    pub avg_conviction: f64,
}

impl SkillPerformance {
    pub fn new(skill_id: &str, skill_name: &str, base_weight: f64) -> Self {
        Self {
            skill_id: skill_id.to_string(),
            skill_name: skill_name.to_string(),
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            win_rate: 0.5, // Start with neutral expectation
            avg_win_pct: 0.0,
            avg_loss_pct: 0.0,
            profit_factor: 1.0,
            current_streak: 0,
            total_pnl: 0.0,
            adjusted_weight: base_weight,
            base_weight,
            last_updated: Utc::now(),
            regime_performance: HashMap::new(),
        }
    }

    /// Record a trade outcome and update all metrics
    pub fn record_trade(&mut self, pnl_pct: f64, regime: &str, conviction: f64) {
        self.total_trades += 1;

        if pnl_pct > 0.0 {
            self.winning_trades += 1;
            self.current_streak = if self.current_streak > 0 {
                self.current_streak + 1
            } else {
                1
            };
            self.avg_win_pct = ((self.avg_win_pct * (self.winning_trades as f64 - 1.0)) + pnl_pct)
                / self.winning_trades as f64;
        } else {
            self.losing_trades += 1;
            self.current_streak = if self.current_streak < 0 {
                self.current_streak - 1
            } else {
                -1
            };
            self.avg_loss_pct = ((self.avg_loss_pct * (self.losing_trades as f64 - 1.0)) + pnl_pct.abs())
                / self.losing_trades as f64;
        }

        self.total_pnl += pnl_pct;
        self.win_rate = if self.total_trades > 0 {
            (self.winning_trades as f64 / self.total_trades as f64) * 100.0
        } else {
            50.0
        };

        self.profit_factor = if self.avg_loss_pct > 0.0 {
            (self.win_rate / 100.0) * self.avg_win_pct
                / ((1.0 - self.win_rate / 100.0).max(0.01) * self.avg_loss_pct.max(0.01))
        } else {
            1.0
        };

        // Update per-regime performance
        let regime_perf = self
            .regime_performance
            .entry(regime.to_string())
            .or_insert(RegimePerformance {
                total_trades: 0,
                winning_trades: 0,
                win_rate: 0.5,
                avg_conviction: 0.0,
            });
        regime_perf.total_trades += 1;
        if pnl_pct > 0.0 {
            regime_perf.winning_trades += 1;
        }
        regime_perf.win_rate =
            (regime_perf.winning_trades as f64 / regime_perf.total_trades as f64) * 100.0;
        regime_perf.avg_conviction =
            ((regime_perf.avg_conviction * (regime_perf.total_trades as f64 - 1.0)) + conviction)
                / regime_perf.total_trades as f64;

        self.last_updated = Utc::now();

        // Apply adaptive weight adjustment
        self.adjust_weight();
    }

    /// Adjust the skill weight based on recent performance using exponential decay
    fn adjust_weight(&mut self) {
        let performance_factor = if self.total_trades < 5 {
            // Not enough data — stay close to base
            1.0
        } else {
            // Map win rate relative to 50% baseline
            // If win rate > 50%, boost weight; if < 50%, reduce
            let deviation = (self.win_rate - 50.0) / 50.0; // -1.0 to 1.0
            let confidence = (self.total_trades as f64).min(100.0) / 100.0; // 0.0 to 1.0
            1.0 + deviation * confidence * 0.5 // Max ±50% adjustment
        };

        // Apply profit factor boost for quality
        let quality_boost = if self.profit_factor > 1.5 {
            1.2
        } else if self.profit_factor > 1.0 {
            1.1
        } else if self.profit_factor < 0.5 {
            0.8
        } else {
            1.0
        };

        // Weighted adjustment — blend toward base to prevent runaway weights
        let raw_weight = self.base_weight * performance_factor * quality_boost;
        self.adjusted_weight = self.base_weight * 0.3 + raw_weight * 0.7;
        self.adjusted_weight = self.adjusted_weight.clamp(self.base_weight * 0.1, self.base_weight * 5.0);
    }
}

// ── Learning Engine ───────────────────────────────────────────────────────

/// The self-learning engine that tracks skill performance, adjusts weights,
/// and provides optimized conviction thresholds per market regime.
pub struct LearningEngine {
    /// Per-skill performance tracking
    skills: HashMap<String, SkillPerformance>,
    /// Open trade records (by trade ID)
    open_trades: HashMap<String, Vec<OpenSkillSignal>>,
    /// Regime-specific optimal conviction thresholds
    regime_conviction_optima: HashMap<String, ConvictionOptimum>,
    /// Global config
    config: LearningConfig,
    /// Trade history for analysis
    trade_history: Vec<SkillTradeRecord>,
    max_history: usize,
}

/// A signal that contributed to an open trade
#[derive(Debug, Clone)]
pub struct OpenSkillSignal {
    pub skill_id: String,
    pub skill_name: String,
    pub direction: SignalDirection,
    pub strength: f64,
    pub confidence: f64,
    pub conviction: f64,
    pub regime: String,
}

/// Learned optimal conviction thresholds per regime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvictionOptimum {
    pub regime: String,
    pub min_conviction: f64,
    pub sample_size: u64,
    pub avg_return_when_above: f64,
    pub avg_return_when_below: f64,
}

/// Configuration for the learning engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningConfig {
    /// Enable/disable adaptive weighting
    pub adaptive_weighting_enabled: bool,
    /// Minimum trades before adaptive weighting kicks in
    pub min_trades_for_weighting: u64,
    /// Enable regime-specific conviction optimization
    pub regime_optimization_enabled: bool,
    /// How many recent trades to keep for analysis
    pub max_trade_history: usize,
    /// How aggressively to adjust weights (0.0 = none, 1.0 = full)
    pub learning_rate: f64,
    /// Decay factor for historical trade influence (0.0 to 1.0)
    pub historical_decay: f64,
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            adaptive_weighting_enabled: true,
            min_trades_for_weighting: 5,
            regime_optimization_enabled: true,
            max_trade_history: 10000,
            learning_rate: 0.3,
            historical_decay: 0.95,
        }
    }
}

impl LearningEngine {
    pub fn new(config: LearningConfig) -> Self {
        let max_history = config.max_trade_history;
        Self {
            skills: HashMap::new(),
            open_trades: HashMap::new(),
            regime_conviction_optima: HashMap::new(),
            config,
            trade_history: Vec::with_capacity(max_history),
            max_history,
        }
    }

    /// Register a skill for tracking
    pub fn register_skill(&mut self, skill_id: &str, skill_name: &str, base_weight: f64) {
        self.skills.entry(skill_id.to_string()).or_insert_with(|| {
            SkillPerformance::new(skill_id, skill_name, base_weight)
        });
    }

    /// Register multiple skills at once
    pub fn register_skills(&mut self, skills: &[(String, String, f64)]) {
        for (id, name, weight) in skills {
            self.register_skill(id, name, *weight);
        }
    }

    /// Record the signals that contributed to opening a trade
    pub fn open_trade(
        &mut self,
        trade_id: &str,
        _symbol: &str,
        signals: Vec<SkillSignal>,
        context: &MarketAnalysisContext,
        regime: &str,
    ) {
        let open_signals: Vec<OpenSkillSignal> = signals
            .iter()
            .map(|s| OpenSkillSignal {
                skill_id: s.skill_id.clone(),
                skill_name: s.skill_name.clone(),
                direction: s.direction.clone(),
                strength: s.strength,
                confidence: s.confidence,
                conviction: context.current_price,
                regime: regime.to_string(),
            })
            .collect();

        self.open_trades.insert(trade_id.to_string(), open_signals);
    }

    /// Close a trade and record outcomes for all contributing skills
    #[allow(clippy::too_many_arguments)]
    pub fn close_trade(
        &mut self,
        trade_id: &str,
        symbol: &str,
        entry_price: f64,
        exit_price: f64,
        pnl_pct: f64,
        regime: &str,
        conviction: f64,
    ) -> Vec<String> {
        let mut updated_skills = Vec::new();

        // Record the trade for each contributing skill
        if let Some(signals) = self.open_trades.remove(trade_id) {
            for signal in &signals {
                let was_correct = match signal.direction {
                    SignalDirection::Bullish => pnl_pct > 0.0,
                    SignalDirection::Bearish => pnl_pct > 0.0, // Short wins
                    SignalDirection::Neutral => true,           // Neutral is always "correct" in a way
                };

                // Update skill performance
                if let Some(perf) = self.skills.get_mut(&signal.skill_id) {
                    perf.record_trade(pnl_pct, regime, conviction);
                    updated_skills.push(signal.skill_id.clone());
                }

                // Create trade record
                let record = SkillTradeRecord {
                    id: uuid::Uuid::new_v4().to_string(),
                    symbol: symbol.to_string(),
                    skill_id: signal.skill_id.clone(),
                    skill_name: signal.skill_name.clone(),
                    direction: format!("{:?}", signal.direction),
                    strength: signal.strength,
                    confidence: signal.confidence,
                    entry_price,
                    exit_price: Some(exit_price),
                    pnl: Some(exit_price - entry_price),
                    pnl_pct: Some(pnl_pct),
                    regime: regime.to_string(),
                    timestamp: Utc::now(),
                    closed_at: Some(Utc::now()),
                    was_correct: Some(was_correct),
                };

                self.trade_history.push(record);
                if self.trade_history.len() > self.max_history {
                    self.trade_history.remove(0);
                }
            }
        }

        // Update regime conviction optima
        if self.config.regime_optimization_enabled {
            let optimum = self
                .regime_conviction_optima
                .entry(regime.to_string())
                .or_insert(ConvictionOptimum {
                    regime: regime.to_string(),
                    min_conviction: 0.5,
                    sample_size: 0,
                    avg_return_when_above: 0.0,
                    avg_return_when_below: 0.0,
                });

            optimum.sample_size += 1;
            let high_conviction = conviction >= optimum.min_conviction;
            if high_conviction {
                optimum.avg_return_when_above = ((optimum.avg_return_when_above
                    * (optimum.sample_size as f64 - 1.0))
                    + pnl_pct)
                    / optimum.sample_size as f64;
            } else {
                optimum.avg_return_when_below = ((optimum.avg_return_when_below
                    * (optimum.sample_size as f64 - 1.0))
                    + pnl_pct)
                    / optimum.sample_size as f64;
            }
        }

        updated_skills
    }

    /// Get the adjusted weight for a skill
    pub fn get_skill_weight(&self, skill_id: &str) -> f64 {
        self.skills
            .get(skill_id)
            .map(|p| {
                if self.config.adaptive_weighting_enabled && p.total_trades >= self.config.min_trades_for_weighting
                {
                    p.adjusted_weight
                } else {
                    p.base_weight
                }
            })
            .unwrap_or(1.0)
    }

    /// Get the suggested min conviction for a given regime
    pub fn suggested_conviction(&self, regime: &str) -> f64 {
        if !self.config.regime_optimization_enabled {
            return 0.55;
        }
        self.regime_conviction_optima
            .get(regime)
            .map(|o| {
                if o.sample_size < 10 {
                    0.55
                } else if o.avg_return_when_above > o.avg_return_when_below {
                    // High conviction trades perform better — raise threshold
                    (o.min_conviction + 0.05).min(0.8)
                } else {
                    // Low conviction trades perform better — lower threshold
                    (o.min_conviction - 0.05).max(0.3)
                }
            })
            .unwrap_or(0.55)
    }

    /// Get performance snapshot for all skills
    pub fn all_skill_performance(&self) -> Vec<SkillPerformance> {
        let mut perfs: Vec<SkillPerformance> = self.skills.values().cloned().collect();
        perfs.sort_by_key(|a| std::cmp::Reverse(a.total_trades));
        perfs
    }

    /// Get performance for a specific skill
    pub fn skill_performance(&self, skill_id: &str) -> Option<&SkillPerformance> {
        self.skills.get(skill_id)
    }

    /// Get top-performing skills (by win rate, min 10 trades)
    pub fn top_skills(&self, limit: usize) -> Vec<SkillPerformance> {
        let mut perfs: Vec<SkillPerformance> = self
            .skills
            .values()
            .filter(|p| p.total_trades >= self.config.min_trades_for_weighting)
            .cloned()
            .collect();
        perfs.sort_by(|a, b| {
            b.win_rate
                .partial_cmp(&a.win_rate)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        perfs.truncate(limit);
        perfs
    }

    /// Get underperforming skills (by win rate, min 5 trades)
    pub fn worst_skills(&self, limit: usize) -> Vec<SkillPerformance> {
        let mut perfs: Vec<SkillPerformance> = self
            .skills
            .values()
            .filter(|p| p.total_trades >= 5)
            .cloned()
            .collect();
        perfs.sort_by(|a, b| {
            a.win_rate
                .partial_cmp(&b.win_rate)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        perfs.truncate(limit);
        perfs
    }

    /// Get regime conviction optima
    pub fn regime_optima(&self) -> HashMap<String, ConvictionOptimum> {
        self.regime_conviction_optima.clone()
    }

    /// Get total trades tracked
    pub fn total_trades(&self) -> u64 {
        self.trade_history.len() as u64
    }

    /// Reset all learning data
    pub fn reset(&mut self) {
        self.skills.clear();
        self.open_trades.clear();
        self.regime_conviction_optima.clear();
        self.trade_history.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_performance_tracking() {
        let mut perf = SkillPerformance::new("rsi", "RSI Skill", 1.0);

        // Record 10 winning trades
        for _ in 0..10 {
            perf.record_trade(2.0, "trending", 0.8);
        }
        assert_eq!(perf.total_trades, 10);
        assert_eq!(perf.winning_trades, 10);
        assert_eq!(perf.win_rate, 100.0);
        assert!(perf.adjusted_weight > perf.base_weight);

        // Record 10 losing trades
        for _ in 0..10 {
            perf.record_trade(-1.0, "ranging", 0.6);
        }
        assert_eq!(perf.total_trades, 20);
        assert_eq!(perf.winning_trades, 10);
        assert_eq!(perf.losing_trades, 10);
        assert_eq!(perf.win_rate, 50.0);
    }

    #[test]
    fn test_learning_engine_register() {
        let mut engine = LearningEngine::new(LearningConfig::default());
        engine.register_skill("rsi", "RSI Skill", 1.0);
        engine.register_skill("macd", "MACD Skill", 1.5);

        assert_eq!(engine.skills.len(), 2);
        assert_eq!(engine.get_skill_weight("rsi"), 1.0);
        assert_eq!(engine.get_skill_weight("macd"), 1.5);
    }
}
