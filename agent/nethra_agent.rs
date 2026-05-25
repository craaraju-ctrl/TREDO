use crate::core::agent_core::AgentCore;
use crate::config::AgentConfig;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct NethraAgent {
    core: Arc<AgentCore>,
}

impl NethraAgent {
    pub async fn new(config: AgentConfig) -> Self {
        let core = Arc::new(AgentCore::new(&config).await);
        println!("🚀 NETHRA Autonomous Trading AI Agent Initialized Successfully!");
        Self { core }
    }

    pub async fn execute_trading_cycle(&self, market_data: serde_json::Value) -> Result<String, Box<dyn std::error::Error>> {
        let result = self.core.run_trading_cycle(market_data).await?;
        Ok(result)
    }
}
