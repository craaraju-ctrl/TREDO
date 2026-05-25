use std::sync::Arc;
use serde::Serialize;

use tredo_core::{AgentProvider, LLMProvider, MarketAnalysisContext, SignalDirection};

use crate::bot::{BotRole, SwarmBot, SwarmBotResult};

// ── Bot Swarm ─────────────────────────────────────────────────────────────

/// A swarm of specialized bots running in parallel.
///
/// Each bot has its own role (Technical, Risk, Portfolio, etc.),
/// its own `AgentProvider` for market analysis, and its own
/// `LLMProvider` for reasoning. All bots run concurrently and
/// their results are aggregated into a unified `SwarmAnalysis`.
#[derive(Debug)]
pub struct BotSwarm {
    bots: Vec<SwarmBot>,
}

impl BotSwarm {
    /// Create an empty bot swarm.
    pub fn new() -> Self {
        Self { bots: Vec::new() }
    }

    /// Create a swarm with a predefined set of bots for the TREDO system.
    ///
    /// This is the recommended constructor — it creates a balanced swarm with
    /// specialized roles, split between local and cloud LLM providers.
    pub fn new_tredo_swarm(
        agent: Arc<dyn AgentProvider>,
        local_llm: Arc<dyn LLMProvider>,
        cloud_llm: Arc<dyn LLMProvider>,
    ) -> Self {
        let bots = vec![
            // Technical Analyst — heavily weighted, 30+ skills (uses local Ollama model)
            SwarmBot::new("tech_01", "Technician Alpha", BotRole::TechnicalAnalyst, agent.clone(), local_llm.clone())
                .with_weight(0.30),
            // Risk Assessor — safety-critical (uses cloud Gemini model for deep reasoning)
            SwarmBot::new("risk_01", "Risk Sentinel", BotRole::RiskAssessor, agent.clone(), cloud_llm.clone())
                .with_weight(0.25),
            // Portfolio Manager — allocation oversight (uses local Ollama model)
            SwarmBot::new("port_01", "Portfolio Steward", BotRole::PortfolioManager, agent.clone(), local_llm.clone())
                .with_weight(0.20),
            // Market Intelligence — volume and momentum (uses local Ollama model)
            SwarmBot::new("mkt_01", "Market Scout", BotRole::MarketIntel, agent.clone(), local_llm.clone())
                .with_weight(0.15),
            // Sentiment Analyst — market psychology (uses local Ollama model)
            SwarmBot::new("sent_01", "Sentiment Oracle", BotRole::SentimentAnalyst, agent.clone(), local_llm.clone())
                .with_weight(0.10),
        ];

        Self { bots }
    }

    /// Add a bot to the swarm.
    pub fn add_bot(&mut self, bot: SwarmBot) {
        self.bots.push(bot);
    }

    /// Remove a bot by ID.
    pub fn remove_bot(&mut self, bot_id: &str) -> Option<SwarmBot> {
        if let Some(pos) = self.bots.iter().position(|b| b.id == bot_id) {
            Some(self.bots.remove(pos))
        } else {
            None
        }
    }

    /// Get a reference to all bots.
    pub fn bots(&self) -> &[SwarmBot] {
        &self.bots
    }

    /// Get bot count.
    pub fn bot_count(&self) -> usize {
        self.bots.len()
    }

    /// Run all bots in parallel and return their results.
    pub async fn run_all(&self, context: &MarketAnalysisContext) -> Vec<SwarmBotResult> {
        let handles: Vec<_> = self.bots.iter().map(|bot| bot.run(context)).collect();
        futures::future::join_all(handles).await
    }

    /// Run all bots and aggregate results into a unified analysis.
    pub async fn analyze(&self, context: &MarketAnalysisContext) -> SwarmAnalysis {
        let results = self.run_all(context).await;

        let mut weighted_conviction = 0.0_f64;
        let mut total_weight = 0.0_f64;
        let mut all_signals = 0usize;
        let mut total_bullish = 0u32;
        let mut total_bearish = 0u32;
        let mut total_neutral = 0u32;

        for result in &results {
            let weight = self.bots.iter()
                .find(|b| b.id == result.bot_id)
                .map(|b| b.weight)
                .unwrap_or(0.25);

            weighted_conviction += weight * result.analysis.overall_conviction;
            total_weight += weight;
            all_signals += result.analysis.signals.len();
            total_bullish += result.analysis.bullish_signals;
            total_bearish += result.analysis.bearish_signals;
            total_neutral += result.analysis.neutral_signals;
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

        SwarmAnalysis {
            symbol: context.symbol.clone(),
            current_price: context.current_price,
            bot_results: results,
            overall_conviction,
            overall_direction,
            total_signals: all_signals,
            bullish_signals: total_bullish,
            bearish_signals: total_bearish,
            neutral_signals: total_neutral,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Get metadata about all bots (for introspection / UI / MCP).
    pub fn bot_info(&self) -> Vec<SwarmBotInfo> {
        self.bots
            .iter()
            .map(|b| SwarmBotInfo {
                id: b.id.clone(),
                name: b.name.clone(),
                role: b.role.clone(),
                weight: b.weight,
                system_prompt: b.system_prompt.clone(),
            })
            .collect()
    }
}

impl Default for BotSwarm {
    fn default() -> Self {
        Self::new()
    }
}

// ── Swarm Analysis ────────────────────────────────────────────────────────

/// Aggregated analysis from all bots in the swarm.
#[derive(Debug, Clone, Serialize)]
pub struct SwarmAnalysis {
    pub symbol: String,
    pub current_price: f64,
    pub bot_results: Vec<SwarmBotResult>,
    pub overall_conviction: f64,
    pub overall_direction: SignalDirection,
    pub total_signals: usize,
    pub bullish_signals: u32,
    pub bearish_signals: u32,
    pub neutral_signals: u32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// ── Swarm Bot Info (introspection) ────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct SwarmBotInfo {
    pub id: String,
    pub name: String,
    pub role: BotRole,
    pub weight: f64,
    pub system_prompt: String,
}
