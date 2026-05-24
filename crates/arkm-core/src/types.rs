use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Error Type ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProviderError {
    NotImplemented(String),
    ConfigurationError(String),
    ConnectionError(String),
    Timeout(String),
    AnalysisError(String),
    LearningError(String),
    ExternalError(String),
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderError::NotImplemented(msg) => write!(f, "Not implemented: {}", msg),
            ProviderError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            ProviderError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            ProviderError::Timeout(msg) => write!(f, "Timeout: {}", msg),
            ProviderError::AnalysisError(msg) => write!(f, "Analysis error: {}", msg),
            ProviderError::LearningError(msg) => write!(f, "Learning error: {}", msg),
            ProviderError::ExternalError(msg) => write!(f, "External error: {}", msg),
        }
    }
}

impl std::error::Error for ProviderError {}

// ── Signal Direction ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SignalDirection {
    Bullish,
    Bearish,
    Neutral,
}

impl SignalDirection {
    pub fn as_f64(&self) -> f64 {
        match self {
            SignalDirection::Bullish => 1.0,
            SignalDirection::Bearish => -1.0,
            SignalDirection::Neutral => 0.0,
        }
    }
}

// ── Skill Signal ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSignal {
    pub skill_id: String,
    pub skill_name: String,
    pub direction: SignalDirection,
    pub strength: f64,
    pub confidence: f64,
    pub details: String,
    pub indicators: HashMap<String, f64>,
    pub time_frame: String,
}

// ── Aggregated Analysis ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedAnalysis {
    pub symbol: String,
    pub current_price: f64,
    pub signals: Vec<SkillSignal>,
    pub overall_conviction: f64,
    pub overall_direction: SignalDirection,
    pub bullish_signals: u32,
    pub bearish_signals: u32,
    pub neutral_signals: u32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// ── Market Data Types ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Clone)]
pub struct MarketAnalysisContext {
    pub symbol: String,
    pub candles: Vec<Candle>,
    pub current_price: f64,
    pub cash_available: f64,
    pub portfolio_value: f64,
    pub exposure: f64,
    pub open_positions: HashMap<String, f64>,
}

// ── Portfolio Snapshot ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioSnapshot {
    pub total_value: f64,
    pub cash: f64,
    pub positions: HashMap<String, f64>,
    pub margin_ratio: f64,
    pub daily_pnl: f64,
}

// ── Skill Error ───────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum SkillError {
    InsufficientData(String),
    ComputationError(String),
    InvalidParameters(String),
}

impl std::fmt::Display for SkillError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillError::InsufficientData(msg) => write!(f, "Insufficient data: {}", msg),
            SkillError::ComputationError(msg) => write!(f, "Computation error: {}", msg),
            SkillError::InvalidParameters(msg) => write!(f, "Invalid parameters: {}", msg),
        }
    }
}

// ── Skill Category ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SkillCategory {
    TechnicalAnalysis,
    RiskAssessment,
    PortfolioAnalysis,
    MarketIntelligence,
}

impl std::fmt::Display for SkillCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillCategory::TechnicalAnalysis => write!(f, "Technical Analysis"),
            SkillCategory::RiskAssessment => write!(f, "Risk Assessment"),
            SkillCategory::PortfolioAnalysis => write!(f, "Portfolio Analysis"),
            SkillCategory::MarketIntelligence => write!(f, "Market Intelligence"),
        }
    }
}

// ── Learning Feedback ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct LearningFeedback {
    pub trade_id: String,
    pub symbol: String,
    pub signals: Vec<SkillSignal>,
    pub entry_price: f64,
    pub exit_price: f64,
    pub pnl: f64,
    pub pnl_pct: f64,
    pub regime: String,
    pub conviction: f64,
}

// ── LLM Params ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct LLMParams {
    pub model: String,
    pub temperature: f64,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f64>,
}
