// tredo-swarm — Bot Swarm System
//
// A multi-agent trading swarm where specialized bots,
// each with an AgentProvider + LLMProvider, work in parallel
// and are coordinated by a central decision-maker.
//
// Architecture:
//   SwarmCoordinator (decision maker + LLM reasoning)
//       ├── AgentProvider (primary analytical brain — 30+ skills)
//       ├── LLMProvider (strategic reasoning — connected to model)
//       └── BotSwarm (specialist bots running in parallel)
//               ├── Technical Analyst (agent + LLM)
//               ├── Risk Assessor (agent + LLM)
//               ├── Portfolio Manager (agent + LLM)
//               ├── Market Intelligence (agent + LLM)
//               └── Sentiment Analyst (agent + LLM)

pub mod bot;
pub mod coordinator;
pub mod swarm;

pub use bot::{BotRole, SwarmBot, SwarmBotResult};
pub use coordinator::{CoordinatedOutcome, SwarmAgentProvider, SwarmCoordinator};
pub use swarm::{BotSwarm, SwarmAnalysis, SwarmBotInfo};
