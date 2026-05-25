use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

use crate::{
    AggregatedAnalysis, LearningFeedback, MarketAnalysisContext, NethraAgent, ProviderError,
};
use tredo_core::AgentProvider;

/// The default Rust-native agent provider.
///
/// Wraps `NethraAgent` and its 30+ technical/risk/portfolio skills
/// to implement the `AgentProvider` trait. This is the primary
/// trading intelligence used by the auto-trading loop.
#[derive(Debug)]
pub struct SkilledAgent {
    nethra: Arc<NethraAgent>,
}

impl SkilledAgent {
    pub fn new(nethra: Arc<NethraAgent>) -> Self {
        Self { nethra }
    }
}

#[async_trait]
impl AgentProvider for SkilledAgent {
    fn provider_name(&self) -> &str {
        "nethra"
    }

    async fn analyze_market(
        &self,
        context: &MarketAnalysisContext,
    ) -> Result<AggregatedAnalysis, ProviderError> {
        Ok(self.nethra.analyze(context).await)
    }

    async fn learn(&self, feedback: &LearningFeedback) -> Result<(), ProviderError> {
        let _ = feedback;
        // Learning is handled by the external LearningEngine in tredo-autotrader.
        // This is a no-op; the learning engine feeds weight updates back via update_weight.
        Ok(())
    }

    async fn list_skills(&self) -> Vec<String> {
        self.nethra.skill_names()
    }

    async fn update_weight(&self, name: &str, weight: f64) {
        self.nethra.update_agent_weight(name, weight).await;
    }

    async fn agent_info(&self) -> Vec<HashMap<String, serde_json::Value>> {
        self.nethra
            .agent_info()
            .await
            .into_iter()
            .map(|info| {
                let mut map = HashMap::new();
                map.insert("id".to_string(), serde_json::Value::String(info.id));
                map.insert("name".to_string(), serde_json::Value::String(info.name));
                map.insert("weight".to_string(), serde_json::json!(info.weight));
                map
            })
            .collect()
    }
}
