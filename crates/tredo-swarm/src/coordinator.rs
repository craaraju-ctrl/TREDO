use async_trait::async_trait;
use serde::Serialize;
use std::sync::Arc;

use tredo_core::{
    AgentProvider, AggregatedAnalysis, LLMProvider, LearningFeedback, MarketAnalysisContext,
    ProviderError, SignalDirection,
};

use crate::bot::SwarmBotResult;
use crate::swarm::{BotSwarm, SwarmAnalysis};

// ── Swarm Coordinator ─────────────────────────────────────────────────────

/// The central decision-maker for the bot swarm.
///
/// The coordinator:
/// 1. Runs all bots in the swarm in parallel
/// 2. Uses its `AgentProvider` as the primary analytical "brain"
/// 3. Uses its `LLMProvider` for strategic reasoning
/// 4. Produces a final coordinated decision
/// 5. Implements `AgentProvider` so it can be plugged into the existing system
///
/// This is the "general" that commands the swarm of specialists.
#[derive(Debug)]
pub struct SwarmCoordinator {
    /// Primary agent for market analysis (decision maker)
    agent: Arc<dyn AgentProvider>,
    /// LLM for strategic reasoning (connected to model)
    llm: Arc<dyn LLMProvider>,
    /// The bot swarm
    swarm: BotSwarm,
    /// System prompt for the coordinator's LLM
    system_prompt: String,
}

impl SwarmCoordinator {
    /// Create a new SwarmCoordinator.
    ///
    /// - `agent`: The primary decision-maker (e.g., SkilledAgent with 30+ skills)
    /// - `llm`: The LLM for strategic reasoning (e.g., GeminiLLM)
    /// - `swarm`: The bot swarm to coordinate
    /// - `registry_name`: Optional name for runtime provider discovery
    pub fn new(agent: Arc<dyn AgentProvider>, llm: Arc<dyn LLMProvider>, swarm: BotSwarm) -> Self {
        let system_prompt = format!(
            "You are Nethra, the central decision-maker and Master Coordinator for an AI trading swarm. \
             You command {} specialized bots. Your role:\n\
             1. Review analyses from all bots\n\
             2. Consider conviction levels and signal counts\n\
             3. Produce a clear trading decision (BUY/SELL/HOLD/SKIP)\n\
             4. Provide a concise reasoning summary\n\n\
             Always be data-driven. Never invent data.",
            swarm.bot_count()
        );

        Self {
            agent,
            llm,
            swarm,
            system_prompt,
        }
    }

    /// Set a custom system prompt for the coordinator's LLM.
    pub fn with_system_prompt(mut self, prompt: &str) -> Self {
        self.system_prompt = prompt.to_string();
        self
    }

    /// Get a reference to the underlying bot swarm.
    pub fn swarm(&self) -> &BotSwarm {
        &self.swarm
    }

    /// Get a mutable reference to the underlying bot swarm.
    pub fn swarm_mut(&mut self) -> &mut BotSwarm {
        &mut self.swarm
    }

    /// Get a reference to the primary decision-maker agent.
    pub fn decision_agent(&self) -> &Arc<dyn AgentProvider> {
        &self.agent
    }

    /// Get a reference to the strategic reasoning LLM.
    pub fn reasoning_llm(&self) -> &Arc<dyn LLMProvider> {
        &self.llm
    }

    /// Full orchestration pipeline:
    /// 1. Run all bots in the swarm
    /// 2. Run primary agent analysis
    /// 3. Produce final coordinated decision with LLM reasoning
    pub async fn orchestrate(&self, context: &MarketAnalysisContext) -> CoordinatedOutcome {
        // Step 1: Run all bots in the swarm in parallel
        let bot_results = self.swarm.run_all(context).await;

        // Step 2: Run primary agent analysis (the core decision maker)
        let primary_analysis = self
            .agent
            .analyze_market(context)
            .await
            .unwrap_or_else(|e| {
                eprintln!("[SwarmCoordinator] Primary agent error: {:?}", e);
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
            });

        // Step 3: Build swarm analysis from already-computed bot results
        //         (avoids double execution — swarm.analyze() would call run_all() again)
        let swarm_analysis = self.build_swarm_analysis_from_results(context, &bot_results);

        // Step 4: Use the LLM to produce final strategic reasoning
        let final_reasoning = self
            .reason_strategic(&primary_analysis, &swarm_analysis, &bot_results)
            .await;

        // Step 5: Determine final decision
        let decision = self.final_decision(&primary_analysis, &swarm_analysis);

        CoordinatedOutcome {
            symbol: context.symbol.clone(),
            current_price: context.current_price,
            primary_analysis,
            swarm_analysis,
            bot_results,
            final_reasoning,
            decision,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Build a `SwarmAnalysis` from existing bot results without re-running bots.
    /// This mirrors the aggregation logic in `BotSwarm::analyze` but uses
    /// pre-computed results to avoid double execution.
    fn build_swarm_analysis_from_results(
        &self,
        context: &MarketAnalysisContext,
        results: &[SwarmBotResult],
    ) -> SwarmAnalysis {
        let mut weighted_conviction = 0.0_f64;
        let mut total_weight = 0.0_f64;
        let mut all_signals = 0usize;
        let mut total_bullish = 0u32;
        let mut total_bearish = 0u32;
        let mut total_neutral = 0u32;

        for result in results {
            let weight = self
                .swarm
                .bots()
                .iter()
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
            bot_results: results.to_vec(),
            overall_conviction,
            overall_direction,
            total_signals: all_signals,
            bullish_signals: total_bullish,
            bearish_signals: total_bearish,
            neutral_signals: total_neutral,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Use the LLM to produce strategic reasoning incorporating bot results.
    async fn reason_strategic(
        &self,
        primary: &AggregatedAnalysis,
        swarm: &SwarmAnalysis,
        results: &[SwarmBotResult],
    ) -> String {
        // Build a summary of bot results
        let bot_summaries: Vec<String> = results
            .iter()
            .map(|r| {
                format!(
                    "[{}] Conviction: {:.2} ({:?}) | Reasoning: {}",
                    r.bot_name,
                    r.analysis.overall_conviction,
                    r.analysis.overall_direction,
                    r.llm_reasoning
                )
            })
            .collect();

        let prompt = format!(
            "NETHRA COORDINATION REPORT\n\
             ==========================\n\n\
             Symbol: {} @ ${:.4}\n\n\
             PRIMARY AGENT ANALYSIS:\n\
             Conviction: {:.2} ({:?})\n\
             Signals: {} bullish, {} bearish, {} neutral\n\n\
             SWARM CONSENSUS:\n\
             Overall: {:.2} ({:?})\n\
             Bot count: {}\n\n\
             BOT RESULTS:\n\
             {}\n\n\
             Produce final decision and 2-3 sentence reasoning.",
            primary.symbol,
            primary.current_price,
            primary.overall_conviction,
            primary.overall_direction,
            primary.bullish_signals,
            primary.bearish_signals,
            primary.neutral_signals,
            swarm.overall_conviction,
            swarm.overall_direction,
            swarm.bot_results.len(),
            bot_summaries.join("\n"),
        );

        match self
            .llm
            .complete(&prompt, Some(&self.system_prompt), None)
            .await
        {
            Ok(response) => response,
            Err(e) => {
                eprintln!("[SwarmCoordinator] LLM reasoning error: {:?}", e);
                format!(
                    "Primary conviction {:.2} ({:?}), swarm consensus {:.2} ({:?}). \
                     {} bullish, {} bearish signals across {} bots.",
                    primary.overall_conviction,
                    primary.overall_direction,
                    swarm.overall_conviction,
                    swarm.overall_direction,
                    primary.bullish_signals,
                    primary.bearish_signals,
                    swarm.bot_results.len()
                )
            }
        }
    }

    /// Determine the final trading decision.
    fn final_decision(&self, primary: &AggregatedAnalysis, swarm: &SwarmAnalysis) -> String {
        // Combine primary agent and swarm consensus
        let combined = primary.overall_conviction * 0.6 + swarm.overall_conviction * 0.4;

        let bullish_count = primary.bullish_signals + swarm.bullish_signals;
        let bearish_count = primary.bearish_signals + swarm.bearish_signals;

        if combined > 0.2 && bullish_count > bearish_count {
            format!(
                "BUY — Conviction {:.1}% | {} bullish vs {} bearish (combined)",
                combined * 100.0,
                bullish_count,
                bearish_count
            )
        } else if combined < -0.2 && bearish_count > bullish_count {
            format!(
                "SELL — Conviction {:.1}% | {} bearish vs {} bullish (combined)",
                combined.abs() * 100.0,
                bearish_count,
                bullish_count
            )
        } else if combined.abs() < 0.05 {
            format!(
                "HOLD — Conviction too low ({:.1}%) | {}:{}:{} (B:N:Be)",
                combined.abs() * 100.0,
                primary.bullish_signals + swarm.bullish_signals,
                primary.neutral_signals + swarm.neutral_signals,
                primary.bearish_signals + swarm.bearish_signals,
            )
        } else {
            format!(
                "SKIP — Mixed signals | Conviction {:.1}% | {} bullish vs {} bearish",
                combined * 100.0,
                bullish_count,
                bearish_count
            )
        }
    }
}

// ── ────────────────────────────────────────────────────────────────────────

/// Final coordinated outcome from the full swarm pipeline.
#[derive(Debug, Clone, Serialize)]
pub struct CoordinatedOutcome {
    pub symbol: String,
    pub current_price: f64,
    pub primary_analysis: AggregatedAnalysis,
    pub swarm_analysis: SwarmAnalysis,
    pub bot_results: Vec<SwarmBotResult>,
    pub final_reasoning: String,
    pub decision: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// ═══════════════════════════════════════════════════════════════════════════
//  AgentProvider Implementation — Swarm as a Pluggable Decision Maker
// ═══════════════════════════════════════════════════════════════════════════

/// Wraps the `SwarmCoordinator` as a pluggable `AgentProvider`.
///
/// This allows the bot swarm to be registered in the PluginRegistry and
/// used anywhere an `AgentProvider` is expected — including as the
/// decision-maker for the `AutoTradingLoop`, MCP tools, and API routes.
///
/// The swarm coordinator uses:
/// - Its inner `AgentProvider` for market analysis (the decision maker)
/// - Its `LLMProvider` for strategic reasoning (connected to model)
/// - Its `BotSwarm` for multi-perspective analysis
#[derive(Debug)]
pub struct SwarmAgentProvider {
    coordinator: SwarmCoordinator,
    provider_name: String,
}

impl SwarmAgentProvider {
    pub fn new(coordinator: SwarmCoordinator, provider_name: &str) -> Self {
        Self {
            coordinator,
            provider_name: provider_name.to_string(),
        }
    }

    /// Access the underlying coordinator.
    pub fn coordinator(&self) -> &SwarmCoordinator {
        &self.coordinator
    }

    /// Access the underlying coordinator mutably.
    pub fn coordinator_mut(&mut self) -> &mut SwarmCoordinator {
        &mut self.coordinator
    }
}

#[async_trait]
impl AgentProvider for SwarmAgentProvider {
    fn provider_name(&self) -> &str {
        &self.provider_name
    }

    /// Full swarm orchestration: runs all bots + primary agent + LLM reasoning.
    async fn analyze_market(
        &self,
        context: &MarketAnalysisContext,
    ) -> Result<AggregatedAnalysis, ProviderError> {
        let outcome = self.coordinator.orchestrate(context).await;
        Ok(outcome.primary_analysis)
    }

    /// Provide orchestrated analysis with full swarm breakdown as JSON.
    async fn analyze_orchestrated(
        &self,
        context: &MarketAnalysisContext,
    ) -> Option<serde_json::Value> {
        let outcome = self.coordinator.orchestrate(context).await;
        Some(serde_json::to_value(outcome).unwrap_or_default())
    }

    /// Forward learning feedback to the inner agent.
    async fn learn(&self, feedback: &LearningFeedback) -> Result<(), ProviderError> {
        // Delegate learning to the inner agent provider so trade outcomes
        // are used for adaptive weight adjustment
        self.coordinator.decision_agent().learn(feedback).await
    }

    /// List skills from all bots and the primary agent.
    async fn list_skills(&self) -> Vec<String> {
        let mut skills = Vec::new();

        // Get skills from the primary agent (delegated via orchestration)
        // Since we don't have direct agent access, we report swarm structure
        for info in self.coordinator.swarm().bot_info() {
            skills.push(format!("swarm:{} ({})", info.id, info.role.label()));
        }

        skills.push("swarm:coordinator (primary decision maker)".to_string());
        skills.push("swarm:llm (strategic reasoning engine)".to_string());

        skills
    }

    /// Report swarm bot info as agent info.
    async fn agent_info(&self) -> Vec<std::collections::HashMap<String, serde_json::Value>> {
        self.coordinator
            .swarm()
            .bot_info()
            .into_iter()
            .map(|info| {
                let mut map = std::collections::HashMap::new();
                map.insert("id".to_string(), serde_json::Value::String(info.id));
                map.insert("name".to_string(), serde_json::Value::String(info.name));
                map.insert(
                    "role".to_string(),
                    serde_json::Value::String(info.role.label().to_string()),
                );
                map.insert("weight".to_string(), serde_json::json!(info.weight));
                map
            })
            .collect()
    }
}
