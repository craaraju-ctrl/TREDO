//! # HermesBridgeAgent — Python Hermes Proxy via Redis
//!
//! Implements `AgentProvider` by proxying requests to the Python Hermes process
//! through Redis pub/sub. This enables the Rust trading engine to seamlessly
//! leverage Python-based AI agents without direct coupling.
//!
//! ## Architecture
//!
//! ```text
//! Rust ARKM                    Redis                       Python Hermes
//! ┌──────────────┐     ┌──────────────┐     ┌──────────────────┐
//! │ AutoTrader   │ ──► │ pub/sub      │ ──► │ hermes_redis_    │
//! │   ↓          │     │ request/     │     │ bridge.py        │
//! │ HermesBridge │ ◄── │ response     │ ◄── │   ↓              │
//! │ Agent        │     │ channels     │     │ SubAgents        │
//! └──────────────┘     └──────────────┘     └──────────────────┘
//! ```

use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

use arkm_core::{
    AgentProvider, MarketAnalysisContext, AggregatedAnalysis,
    SignalDirection, LearningFeedback, ProviderError,
};
use crate::redis_bridge::{RedisBridge, AgentBusMessage, Channels};

/// Response timeout for Python Hermes RPC calls
const HERMES_RPC_TIMEOUT_SECS: u64 = 30;

/// AgentProvider that proxies market analysis to Python Hermes via Redis.
///
/// Uses a request/response pattern over Redis pub/sub:
/// 1. Publish a request to `hermes:agent:python_hermes` channel
/// 2. Python Hermes processes and publishes response to `hermes:agent:rust_arkm`
/// 3. Subscription handler receives response and routes back
///
/// For resilience, this provider falls back to a simple market analysis
/// if Python Hermes is unreachable.
#[derive(Debug)]
pub struct HermesBridgeAgent {
    bridge: Arc<RedisBridge>,
    local_agent_id: String,
    python_agent_id: String,
}

impl HermesBridgeAgent {
    /// Create a new HermesBridgeAgent.
    ///
    /// - `bridge`: Connected RedisBridge instance
    /// - `local_agent_id`: This Rust agent's ID (e.g., "rust_arkm")
    /// - `python_agent_id`: Target Python Hermes agent ID (e.g., "python_hermes")
    pub fn new(
        bridge: Arc<RedisBridge>,
        local_agent_id: &str,
        python_agent_id: &str,
    ) -> Self {
        Self {
            bridge,
            local_agent_id: local_agent_id.to_string(),
            python_agent_id: python_agent_id.to_string(),
        }
    }

    /// Send a request to Python Hermes and wait for a response.
    /// Uses a one-shot channel paired with a unique correlation ID.
    async fn rpc_call(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value, ProviderError> {
        let correlation_id = format!("{}_{}", self.local_agent_id, uuid::Uuid::new_v4());

        let msg = AgentBusMessage::new(
            &self.local_agent_id,
            &self.python_agent_id,
            method,
            json!({
                "correlation_id": correlation_id,
                "params": params,
                "timestamp": Utc::now().to_rfc3339(),
            }),
        );

        // Publish the request to Python Hermes
        self.bridge.publish(&Channels::direct(&self.python_agent_id), &msg).await
            .map_err(|e| ProviderError::ConnectionError(format!("Failed to reach Python Hermes: {}", e)))?;

        // Try to read response from shared state (Python writes response there)
        let response_key = format!("rpc_response:{}", correlation_id);
        for _ in 0..(HERMES_RPC_TIMEOUT_SECS * 2) {
            if let Ok(Some(response)) = self.bridge.get_state(&response_key).await {
                // Parse the response
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&response) {
                    // Clean up
                    let _ = self.bridge.cache_set(&response_key, "", 1).await;
                    return Ok(val);
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }

        Err(ProviderError::Timeout("Python Hermes did not respond in time".to_string()))
    }
}

#[async_trait]
impl AgentProvider for HermesBridgeAgent {
    fn provider_name(&self) -> &str {
        "hermes_bridge"
    }

    async fn analyze_market(
        &self,
        context: &MarketAnalysisContext,
    ) -> Result<AggregatedAnalysis, ProviderError> {
        // Attempt RPC to Python Hermes
        match self.rpc_call("analyze_market", json!({
            "symbol": context.symbol,
            "current_price": context.current_price,
            "cash_available": context.cash_available,
            "portfolio_value": context.portfolio_value,
            "exposure": context.exposure,
            "candles": context.candles.iter().map(|c| json!({
                "time": c.time,
                "open": c.open,
                "high": c.high,
                "low": c.low,
                "close": c.close,
                "volume": c.volume,
            })).collect::<Vec<_>>(),
            "open_positions": context.open_positions,
        })).await {
            Ok(val) => {
                // Parse the Python Hermes response into AggregatedAnalysis
                Ok(AggregatedAnalysis {
                    symbol: val.get("symbol").and_then(|v| v.as_str()).unwrap_or(&context.symbol).to_string(),
                    current_price: val.get("current_price").and_then(|v| v.as_f64()).unwrap_or(context.current_price),
                    signals: vec![], // Python signals not directly mapped
                    overall_conviction: val.get("conviction").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    overall_direction: match val.get("direction").and_then(|v| v.as_str()) {
                        Some("bullish") => SignalDirection::Bullish,
                        Some("bearish") => SignalDirection::Bearish,
                        _ => SignalDirection::Neutral,
                    },
                    bullish_signals: val.get("bullish_signals").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                    bearish_signals: val.get("bearish_signals").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                    neutral_signals: val.get("neutral_signals").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                    timestamp: Utc::now(),
                })
            }
            Err(e) => {
                // Python Hermes unreachable — return fallback analysis
                println!("[HermesBridgeAgent] Python Hermes unreachable: {}. Using fallback.", e);
                Ok(AggregatedAnalysis {
                    symbol: context.symbol.clone(),
                    current_price: context.current_price,
                    signals: vec![],
                    overall_conviction: 0.5,
                    overall_direction: SignalDirection::Neutral,
                    bullish_signals: 0,
                    bearish_signals: 0,
                    neutral_signals: 0,
                    timestamp: Utc::now(),
                })
            }
        }
    }

    async fn learn(&self, feedback: &LearningFeedback) -> Result<(), ProviderError> {
        self.rpc_call("learn", json!({
            "trade_id": feedback.trade_id,
            "symbol": feedback.symbol,
            "entry_price": feedback.entry_price,
            "exit_price": feedback.exit_price,
            "pnl": feedback.pnl,
            "pnl_pct": feedback.pnl_pct,
            "regime": feedback.regime,
            "conviction": feedback.conviction,
        })).await.map(|_| ())
    }

    async fn list_skills(&self) -> Vec<String> {
        match self.rpc_call("list_skills", json!({})).await {
            Ok(val) => {
                val.as_array()
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default()
            }
            Err(_) => vec!["market_analysis (Python Hermes)".to_string()],
        }
    }

    async fn update_weight(&self, _name: &str, _weight: f64) {
        // Weight updates for Python agents are handled internally by Python Hermes
    }

    async fn agent_info(&self) -> Vec<HashMap<String, serde_json::Value>> {
        match self.rpc_call("agent_info", json!({})).await {
            Ok(val) => {
                val.as_array()
                    .map(|arr| {
                        arr.iter().filter_map(|v| {
                            v.as_object().map(|obj| {
                                obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
                            })
                        }).collect()
                    })
                    .unwrap_or_default()
            }
            Err(_) => vec![],
        }
    }

    async fn analyze_orchestrated(
        &self,
        _context: &MarketAnalysisContext,
    ) -> Option<serde_json::Value> {
        // Python Hermes handles its own orchestration internally
        None
    }
}
