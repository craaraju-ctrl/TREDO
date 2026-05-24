//! # AgentRegistry — Sub-agent Registry & Communication Bus
//!
//! Manages agent registration, discovery, and message routing via Redis pub/sub.
//! Both Python Hermes agents and Rust ARKM sub-agents register here.
//!
//! ## Communication Patterns
//!
//! - **Point-to-point**: Direct messages between specific agents
//! - **Broadcast**: Messages to all agents of a type
//! - **Fan-out**: Messages to all agents subscribed to a topic
//! - **Request/Response**: Async request with correlation ID

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

use crate::redis_bridge::{RedisBridge, AgentBusMessage, Channels};

/// Agent type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentType {
    /// Python Hermes master agent
    PythonHermes,
    /// Rust ARKM trading engine
    RustARKM,
    /// Sub-agent owned by Python
    PythonSubAgent(String),
    /// Sub-agent owned by Rust
    RustSubAgent(String),
    /// External tool/service
    External(String),
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentType::PythonHermes => write!(f, "python_hermes"),
            AgentType::RustARKM => write!(f, "rust_arkm"),
            AgentType::PythonSubAgent(role) => write!(f, "python_subagent:{}", role),
            AgentType::RustSubAgent(role) => write!(f, "rust_subagent:{}", role),
            AgentType::External(name) => write!(f, "external:{}", name),
        }
    }
}

/// Agent capability — what an agent can do
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentCapability {
    /// Can analyze market data
    MarketAnalysis,
    /// Can execute trades
    TradeExecution,
    /// Can assess risk
    RiskAssessment,
    /// Can manage portfolio
    PortfolioManagement,
    /// Can provide intelligence data
    Intelligence,
    /// Can process natural language
    NLP,
    /// Can learn from outcomes
    SelfLearning,
    /// Custom capability
    Custom(String),
}

/// Registered agent metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRegistration {
    pub agent_id: String,
    pub agent_type: AgentType,
    pub display_name: String,
    pub description: String,
    pub capabilities: Vec<AgentCapability>,
    pub channels: Vec<String>,           // Pub/sub channels this agent listens to
    pub weight: f64,                     // Importance/priority in ensemble
    pub status: AgentStatus,
    pub last_heartbeat: DateTime<Utc>,
    pub registered_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentStatus {
    Active,
    Idle,
    Busy,
    Error(String),
    Offline,
}

/// Registry statistics
#[derive(Debug, Clone, Serialize)]
pub struct RegistryStats {
    pub total_agents: usize,
    pub active_agents: usize,
    pub python_agents: usize,
    pub rust_agents: usize,
    pub python_subagents: usize,
    pub rust_subagents: usize,
    pub messages_routed: u64,
}

/// Message router for inter-agent communication
pub struct AgentRegistry {
    bridge: Arc<RedisBridge>,
    /// Local agent cache
    agents: Arc<Mutex<HashMap<String, AgentRegistration>>>,
    /// Local agent ID (this instance)
    local_agent_id: String,
    /// Message routing statistics
    messages_routed: std::sync::atomic::AtomicU64,
}

impl AgentRegistry {
    /// Create a new agent registry
    pub fn new(bridge: Arc<RedisBridge>, agent_id: &str) -> Self {
        Self {
            bridge,
            agents: Arc::new(Mutex::new(HashMap::new())),
            local_agent_id: agent_id.to_string(),
            messages_routed: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Register this agent with the system
    pub async fn register(&self, registration: AgentRegistration) -> Result<(), String> {
        let agent_id = registration.agent_id.clone();

        // Store locally
        {
            let mut agents = self.agents.lock().await;
            agents.insert(agent_id.clone(), registration.clone());
        }

        // Publish to Redis (for other agents to discover)
        let msg = AgentBusMessage::broadcast(
            &self.local_agent_id,
            "agent_register",
            serde_json::to_value(&registration)
                .map_err(|e| format!("Serialize error: {}", e))?,
        );

        self.bridge.publish(Channels::GLOBAL, &msg).await?;

        // Store in agent registry hash
        let json = serde_json::to_string(&registration)
            .map_err(|e| format!("Serialize error: {}", e))?;
        self.bridge.set_state(&format!("agent:{}", agent_id), &json).await?;

        info!("[AgentRegistry] Registered agent: {} ({})", registration.display_name, agent_id);
        Ok(())
    }

    /// Discover all registered agents
    pub async fn discover(&self) -> Result<Vec<AgentRegistration>, String> {
        let mut agents = self.agents.lock().await;

        // Fetch all agents from Redis
        let entries = self.bridge.get_all_state().await?;
        for (key, val) in &entries {
            if key.starts_with("agent:") {
                if let Ok(reg) = serde_json::from_str::<AgentRegistration>(val) {
                    agents.insert(reg.agent_id.clone(), reg);
                }
            }
        }

        Ok(agents.values().cloned().collect())
    }

    /// Send a message to a specific agent
    pub async fn send_to(&self, target_agent: &str, msg_type: &str, payload: serde_json::Value) -> Result<u64, String> {
        let msg = AgentBusMessage::new(
            &self.local_agent_id,
            target_agent,
            msg_type,
            payload,
        );

        let channel = Channels::direct(target_agent);
        let count = self.bridge.publish(&channel, &msg).await?;
        self.messages_routed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(count)
    }

    /// Broadcast a message to all agents
    pub async fn broadcast(&self, msg_type: &str, payload: serde_json::Value) -> Result<u64, String> {
        let msg = AgentBusMessage::broadcast(
            &self.local_agent_id,
            msg_type,
            payload,
        );

        let count = self.bridge.publish(Channels::GLOBAL, &msg).await?;
        self.messages_routed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(count)
    }

    /// Send a message to all agents with a specific capability
    pub async fn send_to_capability(&self, capability: &AgentCapability, msg_type: &str, payload: serde_json::Value) -> Result<u64, String> {
        let agents = self.discover().await?;
        let mut total = 0;

        for agent in agents {
            if agent.capabilities.contains(capability) {
                total += self.send_to(&agent.agent_id, msg_type, payload.clone()).await?;
            }
        }

        Ok(total)
    }

    /// Send trade signal to Python Hermes
    pub async fn send_trade_signal(&self, symbol: &str, action: &str, conviction: f64, reasoning: &str) -> Result<(), String> {
        let payload = serde_json::json!({
            "symbol": symbol,
            "action": action,
            "conviction": conviction,
            "reasoning": reasoning,
            "source": "rust_arkm_autotrader",
            "timestamp": Utc::now().to_rfc3339(),
        });

        // Send to Python Hermes directly
        self.send_to("python_hermes", "trade_signal", payload.clone()).await?;

        // Also publish to trade signals channel
        let msg = AgentBusMessage::broadcast("rust_arkm", "trade_signal", payload);
        self.bridge.publish(Channels::TRADE_SIGNALS, &msg).await?;

        self.messages_routed.fetch_add(3, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    /// Send analysis result to Python Hermes (takes a generic JSON-serializable value)
    pub async fn send_analysis(&self, analysis: &serde_json::Value) -> Result<(), String> {
        self.send_to("python_hermes", "analysis_result", analysis.clone()).await?;
        Ok(())
    }

    /// Subscribe to messages from a specific agent
    pub async fn subscribe_to(&self, agent_id: &str) -> Result<(), String> {
        let channel = Channels::direct(agent_id);
        let local_id = self.local_agent_id.clone();
        let agents = self.agents.clone();

        self.bridge.subscribe(&channel, move |msg: AgentBusMessage| {
            let agents = agents.clone();
            let local_id = local_id.clone();
            async move {
                // Only process messages not from self
                if msg.source != local_id {
                    if let Ok(mut agents) = agents.try_lock() {
                        if let Some(agent) = agents.get_mut(&msg.source) {
                            info!("[AgentRegistry] Message from {}: {}", agent.display_name, msg.msg_type);
                        }
                    }
                }
            }
        }).await
    }

    /// Subscribe to the global channel
    pub async fn subscribe_global(&self) -> Result<(), String> {
        let agents = self.agents.clone();
        let local_id = self.local_agent_id.clone();

        self.bridge.subscribe(Channels::GLOBAL, move |msg: AgentBusMessage| {
            let agents = agents.clone();
            let local_id = local_id.clone();
            async move {
                if msg.source != local_id {
                    // Auto-register new agents discovered via broadcast
                    if msg.msg_type == "agent_register" {
                        if let Ok(reg) = serde_json::from_value::<AgentRegistration>(msg.payload) {
                            if let Ok(mut a) = agents.try_lock() {
                                a.insert(reg.agent_id.clone(), reg);
                            }
                        }
                    }
                }
            }
        }).await
    }

    /// Get registry statistics
    pub async fn stats(&self) -> RegistryStats {
        let agents = self.agents.lock().await;
        RegistryStats {
            total_agents: agents.len(),
            active_agents: agents.values().filter(|a| a.status == AgentStatus::Active).count(),
            python_agents: agents.values().filter(|a| a.agent_type == AgentType::PythonHermes).count(),
            rust_agents: agents.values().filter(|a| a.agent_type == AgentType::RustARKM).count(),
            python_subagents: agents.values().filter(|a| matches!(a.agent_type, AgentType::PythonSubAgent(_))).count(),
            rust_subagents: agents.values().filter(|a| matches!(a.agent_type, AgentType::RustSubAgent(_))).count(),
            messages_routed: self.messages_routed.load(std::sync::atomic::Ordering::Relaxed),
        }
    }
}


