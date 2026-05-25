pub mod technical;
pub mod risk;
pub mod portfolio;
pub mod advanced_technical;
pub mod sub_agents;
pub mod skilled_agent;
pub use skilled_agent::SkilledAgent;

// ── Re-export shared types from tredo-core ────────────────────────────────
pub use tredo_core::{
    MarketAnalysisContext, Candle, SignalDirection, SkillSignal, AggregatedAnalysis,
    PortfolioSnapshot, SkillError, SkillCategory, ProviderError, LearningFeedback,
};

use serde::Serialize;

// ── Trading Skill Trait ───────────────────────────────────────────────────

#[async_trait::async_trait]
pub trait TradingSkill: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn category(&self) -> SkillCategory;

    /// Analyze market data and return a signal
    async fn analyze(&self, context: &MarketAnalysisContext) -> Result<SkillSignal, SkillError>;
}

// ── Nethra Agent ──────────────────────────────────────────────────────────

/// NethraAgent now uses the NethraOrchestrator internally to coordinate
/// specialized sub-agents (TechnicalAnalyst, RiskManager, PortfolioManager,
/// MarketDataAgent). It keeps the same public API for backward compatibility.
pub struct NethraAgent {
    orchestrator: tokio::sync::Mutex<sub_agents::NethraOrchestrator>,
    skills: Vec<Box<dyn TradingSkill>>,
}

impl NethraAgent {
    pub fn new() -> Self {
        let skills = vec![
            // Technical Analysis Skills (Core)
            Box::new(technical::RsiSkill::default()) as Box<dyn TradingSkill>,
            Box::new(technical::MacdSkill::default()),
            Box::new(technical::BollingerBandsSkill::default()),
            Box::new(technical::SmaSkill::new(20)),
            Box::new(technical::SmaSkill::new(50)),
            Box::new(technical::EmaSkill::new(12)),
            Box::new(technical::EmaSkill::new(26)),
            Box::new(technical::SupportResistanceSkill::default()),
            Box::new(technical::VolumeAnalysisSkill),
            // Advanced Technical Analysis Skills (16 strategies)
            Box::new(advanced_technical::IchimokuSkill::default()),
            Box::new(advanced_technical::AdxSkill::default()),
            Box::new(advanced_technical::SuperTrendSkill::default()),
            Box::new(advanced_technical::ParabolicSarSkill::default()),
            Box::new(advanced_technical::KeltnerChannelsSkill::default()),
            Box::new(advanced_technical::AroonSkill::default()),
            Box::new(advanced_technical::PivotPointsSkill),
            Box::new(advanced_technical::ChandelierExitSkill::default()),
            Box::new(advanced_technical::WilliamsRSkill::default()),
            Box::new(advanced_technical::ObvSkill),
            Box::new(advanced_technical::ChaikinMoneyFlowSkill::default()),
            Box::new(advanced_technical::StochasticSkill::default()),
            Box::new(advanced_technical::DonchianChannelsSkill::default()),
            Box::new(advanced_technical::HeikinAshiSkill),
            Box::new(advanced_technical::MarketStructureSkill::default()),
            Box::new(advanced_technical::MarketCypherSkill::default()),
            // Risk Assessment Skills
            Box::new(risk::PositionSizingSkill::default()),
            Box::new(risk::ValueAtRiskSkill::default()),
            Box::new(risk::ExposureLimitSkill::default()),
            Box::new(risk::VolatilityAnalysisSkill::default()),
            // Portfolio Analysis Skills
            Box::new(portfolio::DiversificationSkill::default()),
            Box::new(portfolio::PortfolioHealthSkill),
            Box::new(portfolio::CorrelationRiskSkill),
        ];

        Self {
            orchestrator: tokio::sync::Mutex::new(sub_agents::NethraOrchestrator::new()),
            skills,
        }
    }

    /// Run the orchestrator (sub-agents in parallel) for advanced analysis
    pub async fn orchestrate(&self, context: &MarketAnalysisContext) -> sub_agents::OrchestratedResult {
        let orchestrator = self.orchestrator.lock().await;
        orchestrator.orchestrate(context).await
    }

    /// Update an agent weight (used by learning engine)
    pub async fn update_agent_weight(&self, agent_id: &str, weight: f64) {
        let mut orchestrator = self.orchestrator.lock().await;
        orchestrator.update_agent_weight(agent_id, weight);
    }

    pub fn skills_by_category(&self, category: SkillCategory) -> Vec<&dyn TradingSkill> {
        self.skills.iter().map(Box::as_ref).filter(|s| s.category() == category).collect()
    }

    /// Run all applicable skills and produce an aggregated analysis
    pub async fn analyze(&self, context: &MarketAnalysisContext) -> AggregatedAnalysis {
        let mut signals = Vec::new();
        let mut bullish = 0u32;
        let mut bearish = 0u32;
        let mut neutral = 0u32;
        let mut weighted_sum: f64 = 0.0;
        let mut total_weight: f64 = 0.0;

        for skill in &self.skills {
            match skill.analyze(context).await {
                Ok(signal) => {
                    let weight = signal.strength * signal.confidence;
                    weighted_sum += weight * signal.direction.as_f64();
                    total_weight += weight;

                    match signal.direction {
                        SignalDirection::Bullish => bullish += 1,
                        SignalDirection::Bearish => bearish += 1,
                        SignalDirection::Neutral => neutral += 1,
                    }

                    signals.push(signal);
                }
                Err(e) => {
                    eprintln!("[NethraAgent] Skill '{}' error: {}", skill.name(), e);
                }
            }
        }

        let overall_conviction: f64 = if total_weight > 0.0 {
            (weighted_sum / total_weight).clamp(-1.0, 1.0)
        } else {
            0.0
        };

        let overall_direction = if overall_conviction > 0.15 {
            SignalDirection::Bullish
        } else if overall_conviction < -0.15 {
            SignalDirection::Bearish
        } else {
            SignalDirection::Neutral
        };

        AggregatedAnalysis {
            symbol: context.symbol.clone(),
            current_price: context.current_price,
            signals,
            overall_conviction,
            overall_direction,
            bullish_signals: bullish,
            bearish_signals: bearish,
            neutral_signals: neutral,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn skill_count(&self) -> usize {
        self.skills.len()
    }

    pub fn skill_names(&self) -> Vec<String> {
        self.skills.iter().map(|s| format!("{} ({})", s.name(), s.category())).collect()
    }

    /// Get info about all sub-agents
    pub async fn agent_info(&self) -> Vec<sub_agents::AgentInfo> {
        let orchestrator = self.orchestrator.lock().await;
        orchestrator.agent_info()
    }
}

impl std::fmt::Debug for NethraAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NethraAgent")
            .field("skill_count", &self.skills.len())
            .finish()
    }
}

impl Default for NethraAgent {
    fn default() -> Self {
        Self::new()
    }
}
