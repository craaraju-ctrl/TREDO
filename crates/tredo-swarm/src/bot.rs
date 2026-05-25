use std::sync::Arc;
use serde::Serialize;

use tredo_core::{
    AgentProvider, LLMProvider, MarketAnalysisContext, AggregatedAnalysis,
    SignalDirection,
};

// ── Bot Role ──────────────────────────────────────────────────────────────

/// Specialized role for a bot in the swarm.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum BotRole {
    /// Technical analysis specialist (RSI, MACD, Bollinger, etc.)
    TechnicalAnalyst,
    /// Risk assessment specialist (VaR, position sizing, exposure)
    RiskAssessor,
    /// Portfolio management specialist (diversification, allocation)
    PortfolioManager,
    /// Market intelligence analyst (volume, price action, patterns)
    MarketIntel,
    /// Sentiment and news analyst
    SentimentAnalyst,
    /// Macro-economic analyst
    MacroAnalyst,
}

impl BotRole {
    pub fn label(&self) -> &str {
        match self {
            BotRole::TechnicalAnalyst => "Technical Analyst",
            BotRole::RiskAssessor => "Risk Assessor",
            BotRole::PortfolioManager => "Portfolio Manager",
            BotRole::MarketIntel => "Market Intelligence",
            BotRole::SentimentAnalyst => "Sentiment Analyst",
            BotRole::MacroAnalyst => "Macro Analyst",
        }
    }

    pub fn default_system_prompt(&self) -> &str {
        match self {
            BotRole::TechnicalAnalyst => {
                "You are a technical analysis specialist. Focus on chart patterns, indicators, \
                 and price action. Provide clear signal direction and conviction levels."
            }
            BotRole::RiskAssessor => {
                "You are a risk assessment specialist. Evaluate position sizing, volatility, \
                 value-at-risk, and exposure limits. Prioritize capital preservation."
            }
            BotRole::PortfolioManager => {
                "You are a portfolio manager. Assess diversification, correlation risks, \
                 and overall portfolio health. Consider rebalancing and allocation."
            }
            BotRole::MarketIntel => {
                "You are a market intelligence analyst. Detect volume surges, price momentum, \
                 support/resistance levels, and emerging patterns in market data."
            }
            BotRole::SentimentAnalyst => {
                "You are a sentiment analyst. Evaluate market sentiment from volume flow, \
                 positioning, and order book imbalance. Gauge fear vs greed."
            }
            BotRole::MacroAnalyst => {
                "You are a macro-economic analyst. Consider broader market trends, economic \
                 indicators, sector rotation, and inter-market correlations."
            }
        }
    }
}

// ── Bot Result ────────────────────────────────────────────────────────────

/// Result from a single bot in the swarm.
#[derive(Debug, Clone, Serialize)]
pub struct SwarmBotResult {
    pub bot_id: String,
    pub bot_name: String,
    pub role: BotRole,
    pub analysis: AggregatedAnalysis,
    pub llm_reasoning: String,
    pub confidence: f64,
}

// ── Swarm Bot ─────────────────────────────────────────────────────────────

/// A single bot in the trading swarm.
///
/// Each bot has:
/// - An `AgentProvider` for market analysis (30+ skills)
/// - An `LLMProvider` for reasoning and natural language output
/// - A specialized role with system prompt
#[derive(Debug)]
pub struct SwarmBot {
    pub id: String,
    pub name: String,
    pub role: BotRole,
    pub agent: Arc<dyn AgentProvider>,
    pub llm: Arc<dyn LLMProvider>,
    pub system_prompt: String,
    pub weight: f64,
}

impl SwarmBot {
    /// Create a new bot with a given role, agent, and LLM.
    pub fn new(
        id: &str,
        name: &str,
        role: BotRole,
        agent: Arc<dyn AgentProvider>,
        llm: Arc<dyn LLMProvider>,
    ) -> Self {
        let system_prompt = role.default_system_prompt().to_string();

        Self {
            id: id.to_string(),
            name: name.to_string(),
            role,
            agent,
            llm,
            system_prompt,
            weight: 0.25,
        }
    }

    /// Set a custom system prompt (overrides role default).
    pub fn with_system_prompt(mut self, prompt: &str) -> Self {
        self.system_prompt = prompt.to_string();
        self
    }

    /// Set the bot's weight in swarm consensus.
    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight.clamp(0.05, 0.8);
        self
    }

    /// Run market analysis using the agent provider.
    pub async fn analyze(&self, context: &MarketAnalysisContext) -> AggregatedAnalysis {
        self.agent.analyze_market(context).await.unwrap_or_else(|e| {
            eprintln!("[SwarmBot:{}] Agent analysis error: {:?}", self.id, e);
            AggregatedAnalysis {
                symbol: context.symbol.clone(),
                current_price: context.current_price,
                signals: vec![],
                overall_conviction: 0.0,
                overall_direction: SignalDirection::Neutral,
                bullish_signals: 0,
                bearish_signals: 0,
                neutral_signals: 0,
                timestamp: chrono::Utc::now(),
            }
        })
    }

    /// Use the LLM to reason about an analysis and produce a summary.
    pub async fn reason(
        &self,
        context: &MarketAnalysisContext,
        analysis: &AggregatedAnalysis,
    ) -> String {
        let prompt = format!(
            "As {}, analyze this market data:\n\
             Symbol: {}\nCurrent Price: ${:.4}\n\
             Overall Conviction: {:.2}\nDirection: {:?}\n\
             Bullish Signals: {} | Bearish: {} | Neutral: {}\n\
             \nProvide a concise 2-3 sentence analysis with your recommendation.",
            self.role.label(),
            context.symbol,
            context.current_price,
            analysis.overall_conviction,
            analysis.overall_direction,
            analysis.bullish_signals,
            analysis.bearish_signals,
            analysis.neutral_signals,
        );

        match self.llm.complete(&prompt, Some(&self.system_prompt), None).await {
            Ok(response) => response,
            Err(e) => {
                eprintln!("[SwarmBot:{}] LLM reasoning error: {:?}", self.id, e);
                format!(
                    "Conviction: {:.2} ({:?}). {} bullish, {} bearish signals.",
                    analysis.overall_conviction,
                    analysis.overall_direction,
                    analysis.bullish_signals,
                    analysis.bearish_signals,
                )
            }
        }
    }

    /// Full analysis pipeline: analyze ➔ reason ➔ produce result.
    pub async fn run(&self, context: &MarketAnalysisContext) -> SwarmBotResult {
        let analysis = self.analyze(context).await;
        let reasoning = self.reason(context, &analysis).await;
        let confidence = analysis.overall_conviction.abs().max(0.1);

        SwarmBotResult {
            bot_id: self.id.clone(),
            bot_name: self.name.clone(),
            role: self.role.clone(),
            analysis,
            llm_reasoning: reasoning,
            confidence,
        }
    }
}
