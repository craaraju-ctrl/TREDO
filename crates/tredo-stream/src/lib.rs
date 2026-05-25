use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::broadcast;

use tredo_skills::AggregatedAnalysis;
use tredo_types::OrderBookSnapshot;

// ── Stream Message Types ──────────────────────────────────────────────────

/// All possible messages that can be streamed to WebSocket clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum StreamMessage {
    /// System alert / status
    Alert {
        severity: String,
        message: String,
        timestamp: DateTime<Utc>,
    },
    /// Real-time price tick
    PriceTick {
        symbol: String,
        price: f64,
        change_24h: f64,
        volume_24h: f64,
        timestamp: DateTime<Utc>,
    },
    /// Full analysis result from Nethra
    AnalysisResult {
        symbol: String,
        analysis: AggregatedAnalysis,
        timestamp: DateTime<Utc>,
    },
    /// Trade decision from the auto-trader
    TradeDecision {
        symbol: String,
        action: String,
        price: f64,
        quantity: f64,
        conviction: f64,
        regime: String,
        summary: String,
        timestamp: DateTime<Utc>,
    },
    /// Order book snapshot
    OrderBook {
        symbol: String,
        snapshot: OrderBookSnapshot,
        timestamp: DateTime<Utc>,
    },
    /// Performance update from the learning engine
    LearningUpdate {
        symbol: String,
        skill_id: String,
        win_rate: f64,
        adjusted_weight: f64,
        trades_evaluated: u64,
        timestamp: DateTime<Utc>,
    },
    /// Trading state update
    StateUpdate {
        enabled: bool,
        balance: f64,
        open_positions: Vec<String>,
        current_drawdown: f64,
        timestamp: DateTime<Utc>,
    },
    /// Error message for client
    Error {
        code: String,
        message: String,
        timestamp: DateTime<Utc>,
    },
}

impl StreamMessage {
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            StreamMessage::Alert { timestamp, .. }
            | StreamMessage::PriceTick { timestamp, .. }
            | StreamMessage::AnalysisResult { timestamp, .. }
            | StreamMessage::TradeDecision { timestamp, .. }
            | StreamMessage::OrderBook { timestamp, .. }
            | StreamMessage::LearningUpdate { timestamp, .. }
            | StreamMessage::StateUpdate { timestamp, .. }
            | StreamMessage::Error { timestamp, .. } => *timestamp,
        }
    }

    pub fn message_type(&self) -> &str {
        match self {
            StreamMessage::Alert { .. } => "alert",
            StreamMessage::PriceTick { .. } => "price_tick",
            StreamMessage::AnalysisResult { .. } => "analysis_result",
            StreamMessage::TradeDecision { .. } => "trade_decision",
            StreamMessage::OrderBook { .. } => "orderbook",
            StreamMessage::LearningUpdate { .. } => "learning_update",
            StreamMessage::StateUpdate { .. } => "state_update",
            StreamMessage::Error { .. } => "error",
        }
    }
}

// ── Broadcast Hub ─────────────────────────────────────────────────────────

/// A tracked broadcast receiver that decrements the connection count on drop.
pub struct TrackedReceiver {
    rx: broadcast::Receiver<StreamMessage>,
    counter: std::sync::Arc<std::sync::atomic::AtomicU32>,
}

impl TrackedReceiver {
    pub fn new(
        rx: broadcast::Receiver<StreamMessage>,
        counter: std::sync::Arc<std::sync::atomic::AtomicU32>,
    ) -> Self {
        Self { rx, counter }
    }

    pub async fn recv(&mut self) -> Result<StreamMessage, broadcast::error::RecvError> {
        self.rx.recv().await
    }

    pub fn try_recv(&mut self) -> Result<StreamMessage, broadcast::error::TryRecvError> {
        self.rx.try_recv()
    }
}

impl Drop for TrackedReceiver {
    fn drop(&mut self) {
        self.counter
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    }
}

/// Central hub for broadcasting stream messages to all connected WebSocket clients.
/// Maintains a typed broadcast channel and connection counters.
pub struct BroadcastHub {
    tx: broadcast::Sender<StreamMessage>,
    connection_count: std::sync::Arc<std::sync::atomic::AtomicU32>,
}

impl BroadcastHub {
    /// Create a new broadcast hub with the given channel capacity.
    /// Messages are dropped for slow consumers if the buffer is full.
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self {
            tx,
            connection_count: std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0)),
        }
    }

    /// Subscribe to receive all broadcast messages
    pub fn subscribe(&self) -> TrackedReceiver {
        self.connection_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        TrackedReceiver::new(self.tx.subscribe(), self.connection_count.clone())
    }

    /// Broadcast a message to all connected clients
    pub fn broadcast(&self, msg: StreamMessage) -> usize {
        self.tx.send(msg).unwrap_or_default()
    }

    /// Get the number of active subscribers
    pub fn active_connections(&self) -> u32 {
        self.connection_count
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Create common convenience messages
    pub fn alert(&self, severity: &str, message: &str) {
        self.broadcast(StreamMessage::Alert {
            severity: severity.to_string(),
            message: message.to_string(),
            timestamp: Utc::now(),
        });
    }

    pub fn price_tick(&self, symbol: &str, price: f64, change_24h: f64, volume_24h: f64) {
        self.broadcast(StreamMessage::PriceTick {
            symbol: symbol.to_string(),
            price,
            change_24h,
            volume_24h,
            timestamp: Utc::now(),
        });
    }

    pub fn analysis(&self, symbol: &str, analysis: &AggregatedAnalysis) {
        self.broadcast(StreamMessage::AnalysisResult {
            symbol: symbol.to_string(),
            analysis: analysis.clone(),
            timestamp: Utc::now(),
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub fn trade_decision(
        &self,
        symbol: &str,
        action: &str,
        price: f64,
        quantity: f64,
        conviction: f64,
        regime: &str,
        summary: &str,
    ) {
        self.broadcast(StreamMessage::TradeDecision {
            symbol: symbol.to_string(),
            action: action.to_string(),
            price,
            quantity,
            conviction,
            regime: regime.to_string(),
            summary: summary.to_string(),
            timestamp: Utc::now(),
        });
    }

    pub fn error(&self, code: &str, message: &str) {
        self.broadcast(StreamMessage::Error {
            code: code.to_string(),
            message: message.to_string(),
            timestamp: Utc::now(),
        });
    }
}

// ── Stream Metadata ───────────────────────────────────────────────────────

/// Metadata about a single stream channel for monitoring
#[derive(Debug, Clone, Serialize)]
pub struct StreamStats {
    pub total_messages_sent: u64,
    pub active_connections: u32,
    pub channel_capacity: usize,
    pub messages_by_type: HashMap<String, u64>,
    pub uptime_seconds: u64,
}

/// Registry that aggregates multiple BroadcastHubs by symbol for organization
pub struct StreamRegistry {
    hubs: HashMap<String, BroadcastHub>,
    global: BroadcastHub,
    started_at: DateTime<Utc>,
    message_count: std::sync::atomic::AtomicU64,
    message_type_counts: std::sync::Mutex<HashMap<String, u64>>,
}

impl StreamRegistry {
    pub fn new() -> Self {
        Self {
            hubs: HashMap::new(),
            global: BroadcastHub::new(4096),
            started_at: Utc::now(),
            message_count: std::sync::atomic::AtomicU64::new(0),
            message_type_counts: std::sync::Mutex::new(HashMap::new()),
        }
    }

    /// Get or create a symbol-specific hub
    pub fn symbol_hub(&mut self, symbol: &str) -> &BroadcastHub {
        let capacity = 1024;
        self.hubs
            .entry(symbol.to_string())
            .or_insert_with(|| BroadcastHub::new(capacity))
    }

    /// Get a reference to the global hub (all messages)
    pub fn global(&self) -> &BroadcastHub {
        &self.global
    }

    /// Broadcast a message to both the global hub and the symbol-specific hub
    pub fn broadcast(&self, symbol: Option<&str>, msg: StreamMessage) {
        let msg_type = msg.message_type().to_string();
        self.global.broadcast(msg.clone());
        if let Some(sym) = symbol {
            if let Some(hub) = self.hubs.get(sym) {
                hub.broadcast(msg);
            }
        }

        // Track message type count (always)
        if let Ok(mut counts) = self.message_type_counts.lock() {
            *counts.entry(msg_type).or_insert(0) += 1;
        }
        self.message_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Get stream statistics
    pub fn stats(&self) -> StreamStats {
        let counts = self
            .message_type_counts
            .lock()
            .ok()
            .map(|guard| guard.clone())
            .unwrap_or_default();
        StreamStats {
            total_messages_sent: self
                .message_count
                .load(std::sync::atomic::Ordering::Relaxed),
            active_connections: self.global.active_connections(),
            channel_capacity: 4096,
            messages_by_type: counts,
            uptime_seconds: (Utc::now() - self.started_at).num_seconds() as u64,
        }
    }
}

impl Default for StreamRegistry {
    fn default() -> Self {
        Self::new()
    }
}
