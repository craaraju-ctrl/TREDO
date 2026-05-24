use tokio::sync::mpsc;
use dashmap::DashMap;
use std::sync::Arc;
use uuid::Uuid;
use arkm_types::{
    ExecutionCommand, TantraCommand,
    TradeDecision, MarketData,
    RiskEngine,
};

// ── Shared Lock-Free State Cache ──────────────────────────────────────────

#[derive(Clone, Default)]
pub struct StateCache {
    pub balances: Arc<DashMap<String, f64>>,
    pub pending_deductions: Arc<DashMap<String, f64>>,
    pub exposures: Arc<DashMap<String, f64>>,
    pub in_flight_orders: Arc<DashMap<Uuid, TradeDecision>>,
    pub prices: Arc<DashMap<String, f64>>,
}

impl StateCache {
    pub fn new() -> Self {
        Self::default()
    }
}

// ── Execution Engine ─────────────────────────────────────────────────────

pub struct ExecutionEngine {
    rx: mpsc::Receiver<ExecutionCommand>,
    pub cache: StateCache,
    risk_engine: RiskEngine,
    tantra_tx: mpsc::Sender<TantraCommand>,
}

impl ExecutionEngine {
    pub fn new(
        rx: mpsc::Receiver<ExecutionCommand>,
        cache: StateCache,
        risk_engine: RiskEngine,
        tantra_tx: mpsc::Sender<TantraCommand>,
    ) -> Self {
        Self { rx, cache, risk_engine, tantra_tx }
    }

    pub async fn run(mut self) {
        println!("[ExecutionEngine] Fast-path engine running.");
        while let Some(cmd) = self.rx.recv().await {
            match cmd {
                ExecutionCommand::Execute(decision) => self.execute_trade(decision).await,
                ExecutionCommand::UpdateBalance(asset, free) => {
                    self.cache.balances.insert(asset, free);
                }
                ExecutionCommand::UpdatePosition(symbol, exposure) => {
                    self.cache.exposures.insert(symbol, exposure);
                }
                ExecutionCommand::RefundInFlight(order_id, asset, amount) => {
                    self.refund_in_flight(order_id, asset, amount);
                }
                ExecutionCommand::AcknowledgeTrade(order_id) => {
                    self.cache.in_flight_orders.remove(&order_id);
                }
            }
        }
    }

    async fn execute_trade(&self, mut decision: TradeDecision) {
        // Sync latest price into decision
        if let Some(price) = self.cache.prices.get(&decision.symbol) {
            decision.price = *price;
        }

        let market_data = self.build_market_data(&decision);

        // Risk gate
        if !self.risk_engine.verify_sync(&decision, &market_data) {
            let _ = self.tantra_tx.send(TantraCommand::RiskRejected(decision)).await;
            return;
        }

        // Optimistic deduction from USDT balance
        let cost = decision.amount * decision.price;
        let asset = "USDT".to_string();

        self.cache.balances.entry(asset.clone()).and_modify(|b| *b -= cost).or_insert(0.0);
        self.cache.pending_deductions.entry(asset.clone()).and_modify(|p| *p += cost).or_insert(cost);
        self.cache.in_flight_orders.insert(decision.id, decision.clone());

        // Dispatch to exchange in a non-blocking task
        let tantra_tx = self.tantra_tx.clone();
        let cache = self.cache.clone();
        let order_id = decision.id;

        tokio::spawn(async move {
            let symbol = decision.symbol.clone();
            let action = decision.action.clone();
            let amount = decision.amount;
            let price = if decision.price > 0.0 { Some(decision.price) } else { None };

            let result = if symbol.contains('-') {
                arkm_exchange::kucoin::execute_order(&symbol, &action, amount, price).await
            } else {
                arkm_exchange::binance::execute_order(&symbol, &action, amount, price).await
            };

            match result {
                Ok(_) => {
                    cache.in_flight_orders.remove(&order_id);
                    cache.pending_deductions.remove(&asset);
                    let _ = tantra_tx.send(TantraCommand::TradeExecuted(decision)).await;
                }
                Err(e) => {
                    eprintln!("[ExecutionEngine] Private signed order dispatch failed: {}", e);
                    // Refund on failure
                    cache.in_flight_orders.remove(&order_id);
                    cache.balances.entry(asset.clone()).and_modify(|b| *b += cost);
                    cache.pending_deductions.remove(&asset);
                    let _ = tantra_tx.send(TantraCommand::RiskRejected(decision)).await;
                }
            }
        });
    }

    fn build_market_data(&self, decision: &TradeDecision) -> MarketData {
        MarketData {
            symbol: decision.symbol.clone(),
            price: decision.price,
            cash_available: self.cache.balances.get("USDT").map(|v| *v).unwrap_or(0.0),
            exposure: self.cache.exposures.get(&decision.symbol).map(|v| *v).unwrap_or(0.0),
        }
    }

    fn refund_in_flight(&self, order_id: Uuid, asset: String, amount: f64) {
        self.cache.in_flight_orders.remove(&order_id);
        self.cache.balances.entry(asset.clone()).and_modify(|b| *b += amount);
        self.cache.pending_deductions.remove(&asset);
    }
}

