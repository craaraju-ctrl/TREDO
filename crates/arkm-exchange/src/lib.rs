use tokio::sync::mpsc;
use std::sync::Arc;
use dashmap::DashMap;
use arkm_types::*;

pub mod binance;
pub mod kucoin;

#[derive(Clone)]
pub struct ExchangeAdapters {
    pub execution_tx: mpsc::Sender<ExecutionCommand>,
    pub symbol_id_map: Arc<DashMap<String, u16>>,
}

impl ExchangeAdapters {
    pub fn new(execution_tx: mpsc::Sender<ExecutionCommand>) -> Self {
        Self {
            execution_tx,
            symbol_id_map: Arc::new(DashMap::new()),
        }
    }

    pub async fn start(self) {
        // Spawn Binance adapter
        let binance = binance::BinanceAdapter::new(
            self.execution_tx.clone(),
            self.symbol_id_map.clone(),
        );
        tokio::spawn(binance.run());

        // Spawn KuCoin adapter
        let kucoin = kucoin::KucoinAdapter::new(
            self.execution_tx.clone(),
            self.symbol_id_map.clone(),
        );
        tokio::spawn(kucoin.run());
    }
}
