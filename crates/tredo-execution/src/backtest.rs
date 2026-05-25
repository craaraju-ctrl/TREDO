use crate::engine::{ExecutionEngine, StateCache};
use tokio::sync::mpsc;
use tredo_types::{
    DecisionStatus, ExecutionCommand, OrderBookSnapshot, RiskEngine, TantraCommand, TradeDecision,
};
use uuid::Uuid;

pub async fn run_backtest(historical_data: Vec<OrderBookSnapshot>) {
    let (tx, rx) = mpsc::channel(1000);
    let (tantra_tx, _) = mpsc::channel::<TantraCommand>(100);

    let cache = StateCache::new();
    cache.balances.insert("USDT".to_string(), 10000.0);

    let engine = ExecutionEngine::new(rx, cache.clone(), RiskEngine::new(), tantra_tx);

    tokio::spawn(engine.run());

    for snapshot in historical_data {
        if snapshot.bids.is_empty() {
            continue;
        }

        let price = snapshot.bids[0].price;
        let _ = tx
            .send(ExecutionCommand::UpdateBalance("USDT".to_string(), 10000.0))
            .await;

        let decision = TradeDecision {
            id: Uuid::new_v4(),
            symbol: snapshot.symbol.clone(),
            action: "BUY".to_string(),
            amount: 0.05,
            price,
            conviction: 0.85,
            reasoning: "Backtest signal confirmed".to_string(),
            status: DecisionStatus::Pending,
            timestamp: chrono::Utc::now(),
        };

        let _ = tx.send(ExecutionCommand::Execute(decision)).await;
    }

    // Allow final tasks to flush
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    println!(
        "✅ Backtest completed. Final USDT balance: {:?}",
        cache.balances.get("USDT").map(|v| *v)
    );
}
