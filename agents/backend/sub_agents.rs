use crate::*;
use std::collections::HashMap;

// ── Sub-Agent Trait ───────────────────────────────────────────────────────

/// A specialized sub-agent that owns a set of related skills and produces
/// a consolidated analysis for its domain.
#[async_trait::async_trait]
pub trait SubAgent: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn category(&self) -> SkillCategory;

    /// Run all owned skills and return a consolidated sub-analysis
    async fn analyze(&self, context: &MarketAnalysisContext) -> SubAgentResult;

    /// Get names of all owned skills
    fn skill_ids(&self) -> Vec<String>;
}

/// Consolidated result from a single sub-agent
#[derive(Debug, Clone, Serialize)]
pub struct SubAgentResult {
    pub agent_id: String,
    pub agent_name: String,
    pub category: SkillCategory,
    pub signals: Vec<SkillSignal>,
    pub conviction: f64,       // -1.0 to 1.0
    pub direction: SignalDirection,
    pub confidence: f64,       // 0.0 to 1.0 — how confident the agent is
    pub summary: String,
    pub processing_time_ms: f64,
}

// ── Technical Analyst ─────────────────────────────────────────────────────

/// Sub-agent specializing in technical analysis
pub struct TechnicalAnalyst {
    skills: Vec<Box<dyn TradingSkill>>,
}

impl TechnicalAnalyst {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for TechnicalAnalyst {
    fn default() -> Self {
        let skills = vec![
            Box::new(crate::technical::RsiSkill::default()) as Box<dyn TradingSkill>,
            Box::new(crate::technical::MacdSkill::default()),
            Box::new(crate::technical::BollingerBandsSkill::default()),
            Box::new(crate::technical::SmaSkill::new(20)),
            Box::new(crate::technical::SmaSkill::new(50)),
            Box::new(crate::technical::EmaSkill::new(12)),
            Box::new(crate::technical::EmaSkill::new(26)),
            Box::new(crate::technical::SupportResistanceSkill::default()),
            Box::new(crate::technical::VolumeAnalysisSkill),
            Box::new(crate::advanced_technical::IchimokuSkill::default()),
            Box::new(crate::advanced_technical::AdxSkill::default()),
            Box::new(crate::advanced_technical::SuperTrendSkill::default()),
            Box::new(crate::advanced_technical::ParabolicSarSkill::default()),
            Box::new(crate::advanced_technical::KeltnerChannelsSkill::default()),
            Box::new(crate::advanced_technical::AroonSkill::default()),
            Box::new(crate::advanced_technical::PivotPointsSkill),
            Box::new(crate::advanced_technical::ChandelierExitSkill::default()),
            Box::new(crate::advanced_technical::WilliamsRSkill::default()),
            Box::new(crate::advanced_technical::ObvSkill),
            Box::new(crate::advanced_technical::ChaikinMoneyFlowSkill::default()),
            Box::new(crate::advanced_technical::StochasticSkill::default()),
            Box::new(crate::advanced_technical::DonchianChannelsSkill::default()),
            Box::new(crate::advanced_technical::HeikinAshiSkill),
            Box::new(crate::advanced_technical::MarketStructureSkill::default()),
            Box::new(crate::advanced_technical::MarketCypherSkill::default()),
        ];

        Self { skills }
    }
}

#[async_trait::async_trait]
impl SubAgent for TechnicalAnalyst {
    fn id(&self) -> &str { "technical_analyst" }
    fn name(&self) -> &str { "Technical Analyst" }
    fn description(&self) -> &str { "Performs comprehensive technical analysis using 25+ indicators across momentum, trend, volume, volatility, and pattern recognition" }
    fn category(&self) -> SkillCategory { SkillCategory::TechnicalAnalysis }

    async fn analyze(&self, context: &MarketAnalysisContext) -> SubAgentResult {
        let start = std::time::Instant::now();
        let mut signals = Vec::new();
        let mut weighted_sum = 0.0_f64;
        let mut total_weight = 0.0_f64;

        for skill in &self.skills {
            match skill.analyze(context).await {
                Ok(signal) => {
                    let weight = signal.strength * signal.confidence;
                    weighted_sum += weight * signal.direction.as_f64();
                    total_weight += weight;
                    signals.push(signal);
                }
                Err(e) => {
                    eprintln!("[TechnicalAnalyst] Skill '{}' error: {}", skill.name(), e);
                }
            }
        }

        let conviction = if total_weight > 0.0 {
            (weighted_sum / total_weight).clamp(-1.0, 1.0)
        } else {
            0.0
        };

        let (direction, summary) = Self::summarize(conviction, &signals);

        SubAgentResult {
            agent_id: self.id().to_string(),
            agent_name: self.name().to_string(),
            category: self.category(),
            signals,
            conviction,
            direction,
            confidence: total_weight / (self.skills.len() as f64).max(1.0),
            summary,
            processing_time_ms: start.elapsed().as_secs_f64() * 1000.0,
        }
    }

    fn skill_ids(&self) -> Vec<String> {
        self.skills.iter().map(|s| s.id().to_string()).collect()
    }
}

impl TechnicalAnalyst {
    fn summarize(conviction: f64, signals: &[SkillSignal]) -> (SignalDirection, String) {
        let bullish = signals.iter().filter(|s| s.direction == SignalDirection::Bullish).count();
        let bearish = signals.iter().filter(|s| s.direction == SignalDirection::Bearish).count();

        let direction = if conviction > 0.15 {
            SignalDirection::Bullish
        } else if conviction < -0.15 {
            SignalDirection::Bearish
        } else {
            SignalDirection::Neutral
        };

        (
            direction.clone(),
            format!(
                "Technical: {}/{} bullish (conviction {:.1}%). {} indicators agree.",
                bullish,
                signals.len(),
                conviction * 100.0,
                match &direction {
                    SignalDirection::Bullish => bullish,
                    _ => bearish,
                }
            ),
        )
    }
}

// ── Risk Manager ──────────────────────────────────────────────────────────

/// Sub-agent specializing in risk assessment
pub struct RiskManager {
    skills: Vec<Box<dyn TradingSkill>>,
}

impl RiskManager {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for RiskManager {
    fn default() -> Self {
        let skills = vec![
            Box::new(crate::risk::PositionSizingSkill::default()) as Box<dyn TradingSkill>,
            Box::new(crate::risk::ValueAtRiskSkill::default()),
            Box::new(crate::risk::ExposureLimitSkill::default()),
            Box::new(crate::risk::VolatilityAnalysisSkill::default()),
        ];
        Self { skills }
    }
}

#[async_trait::async_trait]
impl SubAgent for RiskManager {
    fn id(&self) -> &str { "risk_manager" }
    fn name(&self) -> &str { "Risk Manager" }
    fn description(&self) -> &str { "Assesses portfolio risk, position sizing, volatility, and exposure limits" }
    fn category(&self) -> SkillCategory { SkillCategory::RiskAssessment }

    async fn analyze(&self, context: &MarketAnalysisContext) -> SubAgentResult {
        let start = std::time::Instant::now();
        let mut signals = Vec::new();
        let mut weighted_sum = 0.0_f64;
        let mut total_weight = 0.0_f64;

        for skill in &self.skills {
            match skill.analyze(context).await {
                Ok(signal) => {
                    let weight = signal.strength * signal.confidence;
                    weighted_sum += weight * signal.direction.as_f64();
                    total_weight += weight;
                    signals.push(signal);
                }
                Err(e) => {
                    eprintln!("[RiskManager] Skill '{}' error: {}", skill.name(), e);
                }
            }
        }

        let risk_score = if total_weight > 0.0 {
            (-(weighted_sum / total_weight)).clamp(0.0, 1.0) // Invert: more risk = more negative
        } else {
            0.5
        };

        SubAgentResult {
            agent_id: self.id().to_string(),
            agent_name: self.name().to_string(),
            category: self.category(),
            signals,
            conviction: weighted_sum,
            direction: if risk_score > 0.6 { SignalDirection::Bearish } else if risk_score < 0.3 { SignalDirection::Bullish } else { SignalDirection::Neutral },
            confidence: total_weight / (self.skills.len() as f64).max(1.0),
            summary: format!("Risk score: {:.1}% — {}",
                risk_score * 100.0,
                if risk_score > 0.6 { "HIGH RISK: Reduce exposure recommended" }
                else if risk_score > 0.3 { "MODERATE RISK: Standard position sizing" }
                else { "LOW RISK: Favorable for increased exposure" }
            ),
            processing_time_ms: start.elapsed().as_secs_f64() * 1000.0,
        }
    }

    fn skill_ids(&self) -> Vec<String> {
        self.skills.iter().map(|s| s.id().to_string()).collect()
    }
}

// ── Portfolio Manager ──────────────────────────────────────────────────────

/// Sub-agent specializing in portfolio analysis
pub struct PortfolioManager {
    skills: Vec<Box<dyn TradingSkill>>,
}

impl PortfolioManager {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for PortfolioManager {
    fn default() -> Self {
        let skills = vec![
            Box::new(crate::portfolio::DiversificationSkill::default()) as Box<dyn TradingSkill>,
            Box::new(crate::portfolio::PortfolioHealthSkill),
            Box::new(crate::portfolio::CorrelationRiskSkill),
        ];
        Self { skills }
    }
}

#[async_trait::async_trait]
impl SubAgent for PortfolioManager {
    fn id(&self) -> &str { "portfolio_manager" }
    fn name(&self) -> &str { "Portfolio Manager" }
    fn description(&self) -> &str { "Manages portfolio allocation, diversification, and overall health" }
    fn category(&self) -> SkillCategory { SkillCategory::PortfolioAnalysis }

    async fn analyze(&self, context: &MarketAnalysisContext) -> SubAgentResult {
        let start = std::time::Instant::now();
        let mut signals = Vec::new();
        let mut weighted_sum = 0.0_f64;
        let mut total_weight = 0.0_f64;

        for skill in &self.skills {
            match skill.analyze(context).await {
                Ok(signal) => {
                    let weight = signal.strength * signal.confidence;
                    weighted_sum += weight * signal.direction.as_f64();
                    total_weight += weight;
                    signals.push(signal);
                }
                Err(e) => {
                    eprintln!("[PortfolioManager] Skill '{}' error: {}", skill.name(), e);
                }
            }
        }

        let health_score = if total_weight > 0.0 {
            (weighted_sum / total_weight).clamp(-1.0, 1.0)
        } else {
            0.0
        };

        SubAgentResult {
            agent_id: self.id().to_string(),
            agent_name: self.name().to_string(),
            category: self.category(),
            signals,
            conviction: health_score,
            direction: if health_score > 0.2 { SignalDirection::Bullish } else if health_score < -0.2 { SignalDirection::Bearish } else { SignalDirection::Neutral },
            confidence: total_weight / (self.skills.len() as f64).max(1.0),
            summary: format!("Portfolio health: {:.1}% — {}",
                (health_score + 1.0) * 50.0,
                if health_score > 0.3 { "Well diversified, room for new positions" }
                else if health_score > -0.3 { "Adequate diversification, standard allocation" }
                else { "Over-concentrated, consider rebalancing" }
            ),
            processing_time_ms: start.elapsed().as_secs_f64() * 1000.0,
        }
    }

    fn skill_ids(&self) -> Vec<String> {
        self.skills.iter().map(|s| s.id().to_string()).collect()
    }
}

// ── Market Data Agent ──────────────────────────────────────────────────────

/// Sub-agent that processes raw market data and provides context-aware analysis
pub struct MarketDataAgent;

#[async_trait::async_trait]
impl SubAgent for MarketDataAgent {
    fn id(&self) -> &str { "market_data_agent" }
    fn name(&self) -> &str { "Market Data Agent" }
    fn description(&self) -> &str { "Processes raw market data, detects patterns, and provides contextual market intelligence" }
    fn category(&self) -> SkillCategory { SkillCategory::MarketIntelligence }

    async fn analyze(&self, context: &MarketAnalysisContext) -> SubAgentResult {
        let start = std::time::Instant::now();

        if context.candles.is_empty() {
            return SubAgentResult {
                agent_id: self.id().to_string(),
                agent_name: self.name().to_string(),
                category: self.category(),
                signals: vec![],
                conviction: 0.0,
                direction: SignalDirection::Neutral,
                confidence: 0.0,
                summary: "No market data available for analysis".to_string(),
                processing_time_ms: start.elapsed().as_secs_f64() * 1000.0,
            };
        }

        let mut signals = Vec::new();
        let candles = &context.candles;
        let len = candles.len();

        // Price action analysis
        let last = &candles[len - 1];
        let prev = if len >= 2 { &candles[len - 2] } else { last };

        // Detect momentum
        let price_change = if prev.close > 0.0 {
            (last.close - prev.close) / prev.close * 100.0
        } else {
            0.0
        };

        // Volume surge detection
        let avg_volume: f64 = candles.iter().map(|c| c.volume).sum::<f64>() / len as f64;
        let volume_surge = if avg_volume > 0.0 { last.volume / avg_volume } else { 1.0 };

        // Range analysis
        let highs: Vec<f64> = candles.iter().map(|c| c.high).collect();
        let lows: Vec<f64> = candles.iter().map(|c| c.low).collect();
        let max_high = highs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min_low = lows.iter().cloned().fold(f64::INFINITY, f64::min);
        let range = if min_low > 0.0 { (max_high - min_low) / min_low * 100.0 } else { 0.0 };

        // Intrabar volatility
        let body = (last.close - last.open).abs();
        let candle_range = (last.high - last.low).abs();
        let volatility_ratio = if candle_range > 0.0 { body / candle_range } else { 0.5 };

        // Determine direction
        let (direction, strength, raw_conviction) = if price_change > 1.0 && volume_surge > 1.5 {
            (SignalDirection::Bullish, 0.7, 0.6)
        } else if price_change < -1.0 && volume_surge > 1.5 {
            (SignalDirection::Bearish, 0.7, 0.6)
        } else if price_change > 0.5 {
            (SignalDirection::Bullish, 0.4, 0.3)
        } else if price_change < -0.5 {
            (SignalDirection::Bearish, 0.4, 0.3)
        } else {
            (SignalDirection::Neutral, 0.3, 0.5)
        };

        let conviction_val = match &direction {
            SignalDirection::Bullish => raw_conviction,
            SignalDirection::Bearish => -raw_conviction,
            SignalDirection::Neutral => 0.0,
        };

        let signal = SkillSignal {
            skill_id: "market_data_momentum".to_string(),
            skill_name: "Price Momentum".to_string(),
            direction: direction.clone(),
            strength,
            confidence: 0.7,
            details: format!(
                "Price change: {:.2}%, Volume surge: {:.2}x, Range: {:.2}%",
                price_change, volume_surge, range
            ),
            indicators: HashMap::from([
                ("price_change_pct".to_string(), price_change),
                ("volume_surge_ratio".to_string(), volume_surge),
                ("range_pct".to_string(), range),
                ("volatility_ratio".to_string(), volatility_ratio),
            ]),
            time_frame: "current".to_string(),
        };

        signals.push(signal);

        SubAgentResult {
            agent_id: self.id().to_string(),
            agent_name: self.name().to_string(),
            category: self.category(),
            signals,
            conviction: conviction_val,
            direction,
            confidence: 0.7,
            summary: format!(
                "Market: {} change ({:.1}%), vol {:.1}x — {}",
                if price_change > 0.0 { "+" } else { "" },
                price_change,
                volume_surge,
                if volume_surge > 1.5 { "abnormal volume detected" } else { "normal flow" }
            ),
            processing_time_ms: start.elapsed().as_secs_f64() * 1000.0,
        }
    }

    fn skill_ids(&self) -> Vec<String> {
        vec!["market_data_momentum".to_string()]
    }
}

// ── Orchestrator Agent ─────────────────────────────────────────────────────

/// Master orchestrator that coordinates all sub-agents, aggregates their
/// analyses, and produces a unified trading signal.
pub struct NethraOrchestrator {
    agents: Vec<Box<dyn SubAgent>>,
    agent_weights: HashMap<String, f64>,
}

impl NethraOrchestrator {
    pub fn new() -> Self {
        let agents: Vec<Box<dyn SubAgent>> = vec![
            Box::new(TechnicalAnalyst::new()),
            Box::new(RiskManager::new()),
            Box::new(PortfolioManager::new()),
            Box::new(MarketDataAgent),
        ];

        // Default weights — can be overridden by the learning engine
        let mut agent_weights = HashMap::new();
        agent_weights.insert("technical_analyst".to_string(), 0.40);
        agent_weights.insert("risk_manager".to_string(), 0.25);
        agent_weights.insert("portfolio_manager".to_string(), 0.20);
        agent_weights.insert("market_data_agent".to_string(), 0.15);

        Self { agents, agent_weights }
    }

    /// Update weights for specific agents (called by learning engine)
    pub fn update_agent_weight(&mut self, agent_id: &str, weight: f64) {
        self.agent_weights
            .entry(agent_id.to_string())
            .and_modify(|w| *w = weight.clamp(0.05, 0.8));
    }

    /// Run all sub-agents in parallel and aggregate results
    pub async fn orchestrate(&self, context: &MarketAnalysisContext) -> OrchestratedResult {
        let mut handles = Vec::new();

        for agent in &self.agents {
            handles.push(agent.analyze(context));
        }

        let results: Vec<SubAgentResult> = futures::future::join_all(handles).await;

        let mut all_signals = Vec::new();
        let mut weighted_conviction = 0.0_f64;
        let mut total_weight = 0.0_f64;
        let mut agent_results = Vec::new();

        for result in results {
            let weight = self.agent_weights.get(&result.agent_id).copied().unwrap_or(0.25);
            weighted_conviction += weight * result.conviction;
            total_weight += weight;
            all_signals.extend(result.signals.clone());
            agent_results.push(result);
        }

        let overall_conviction = if total_weight > 0.0 {
            (weighted_conviction / total_weight).clamp(-1.0, 1.0)
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

        let bullish = all_signals.iter().filter(|s| s.direction == SignalDirection::Bullish).count() as u32;
        let bearish = all_signals.iter().filter(|s| s.direction == SignalDirection::Bearish).count() as u32;
        let neutral = all_signals.iter().filter(|s| s.direction == SignalDirection::Neutral).count() as u32;

        OrchestratedResult {
            symbol: context.symbol.clone(),
            current_price: context.current_price,
            agent_results,
            all_signals,
            overall_conviction,
            overall_direction,
            bullish_signals: bullish,
            bearish_signals: bearish,
            neutral_signals: neutral,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Get a list of all agents with their metadata
    pub fn agent_info(&self) -> Vec<AgentInfo> {
        self.agents
            .iter()
            .map(|a| AgentInfo {
                id: a.id().to_string(),
                name: a.name().to_string(),
                description: a.description().to_string(),
                category: a.category(),
                weight: self.agent_weights.get(a.id()).copied().unwrap_or(0.25),
                skill_count: a.skill_ids().len(),
            })
            .collect()
    }

    /// Get total skill count across all agents
    pub fn total_skill_count(&self) -> usize {
        self.agents.iter().map(|a| a.skill_ids().len()).sum()
    }
}

impl Default for NethraOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

/// Metadata about a sub-agent
#[derive(Debug, Clone, Serialize)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: SkillCategory,
    pub weight: f64,
    pub skill_count: usize,
}

/// Final orchestrated result from Nethra
#[derive(Debug, Clone, Serialize)]
pub struct OrchestratedResult {
    pub symbol: String,
    pub current_price: f64,
    pub agent_results: Vec<SubAgentResult>,
    pub all_signals: Vec<SkillSignal>,
    pub overall_conviction: f64,
    pub overall_direction: SignalDirection,
    pub bullish_signals: u32,
    pub bearish_signals: u32,
    pub neutral_signals: u32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
