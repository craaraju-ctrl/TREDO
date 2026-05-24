use super::*;
use tokio::time::{interval, Duration};
use serde::Deserialize;

#[derive(Deserialize)]
struct BinanceExchangeInfo {
    symbols: Vec<BinanceSymbol>,
}

#[derive(Deserialize)]
struct BinanceSymbol {
    symbol: String,
    status: String,
}

pub struct BinanceAdapter {
    execution_tx: mpsc::Sender<ExecutionCommand>,
    symbol_id_map: Arc<DashMap<String, u16>>,
}

impl BinanceAdapter {
    pub fn new(
        execution_tx: mpsc::Sender<ExecutionCommand>,
        symbol_id_map: Arc<DashMap<String, u16>>,
    ) -> Self {
        Self {
            execution_tx,
            symbol_id_map,
        }
    }

    pub async fn run(self) {
        println!("[BinanceAdapter] Starting dynamic token/coin sync...");
        
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("ARKM-Sethu-Exchange-Core/1.0.0")
            .build();

        match client {
            Ok(cli) => {
                match cli.get("https://api.binance.com/api/v3/exchangeInfo").send().await {
                    Ok(resp) => {
                        if let Ok(info) = resp.json::<BinanceExchangeInfo>().await {
                            let mut count = 0;
                            for (idx, sym) in info.symbols.iter().filter(|s| s.status == "TRADING").enumerate() {
                                self.symbol_id_map.insert(sym.symbol.clone(), idx as u16);
                                count += 1;
                            }
                            println!("[BinanceAdapter] Successfully synced {} active tokens/coins from Binance Exchange API!", count);
                        } else {
                            println!("[BinanceAdapter] [Warning] Failed to deserialize Binance ExchangeInfo response. Falling back to default list.");
                            self.load_defaults();
                        }
                    }
                    Err(e) => {
                        println!("[BinanceAdapter] [Warning] REST call to Binance exchangeInfo failed: {}. Using default fallback list.", e);
                        self.load_defaults();
                    }
                }
            }
            Err(_) => {
                self.load_defaults();
            }
        }

        // Listen Key renewal task simulation (every 60s)
        let _renew_tx = self.execution_tx.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
            }
        });

        // User Data Stream simulation
        let mut interval = interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            let _ = self.execution_tx.send(ExecutionCommand::UpdateBalance("USDT".to_string(), 10000.0)).await;
        }
    }

    fn load_defaults(&self) {
        let defaults = ["BTCUSDT", "ETHUSDT", "SOLUSDT", "BNBUSDT", "ADAUSDT", "XRPUSDT", "DOGEUSDT"];
        for (idx, sym) in defaults.iter().enumerate() {
            self.symbol_id_map.insert(sym.to_string(), idx as u16);
        }
        println!("[BinanceAdapter] Loaded {} default fallback symbols.", defaults.len());
    }
}

use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub async fn execute_order(symbol: &str, side: &str, amount: f64, price: Option<f64>) -> Result<(), String> {
    let api_key = std::env::var("BINANCE_API_KEY").unwrap_or_default();
    let secret_key = std::env::var("BINANCE_SECRET_KEY").unwrap_or_default();
    if api_key.is_empty() || secret_key.is_empty() {
        return Err("Binance API keys not set in environment.".to_string());
    }

    let client = reqwest::Client::builder()
        .user_agent("ARKM-Exchange-Core/1.0.0")
        .build()
        .map_err(|e| e.to_string())?;

    let timestamp = chrono::Utc::now().timestamp_millis();
    let order_type = if price.is_some() { "LIMIT" } else { "MARKET" };
    
    let mut query = format!(
        "symbol={}&side={}&type={}&quantity={:.6}&timestamp={}&recvWindow=5000",
        symbol, side, order_type, amount, timestamp
    );
    if let Some(p) = price {
        query.push_str(&format!("&price={:.2}&timeInForce=GTC", p));
    }

    let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes())
        .map_err(|_| "Failed to create HMAC key".to_string())?;
    mac.update(query.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());
    
    let url = format!("https://api.binance.com/api/v3/order?{}&signature={}", query, signature);

    println!("[BinanceAdapter] Dispatching signed private trade: POST https://api.binance.com/api/v3/order");
    let resp = client.post(&url)
        .header("X-MBX-APIKEY", &api_key)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    if status.is_success() {
        println!("[BinanceAdapter] Private trade successfully executed: {}", body);
        Ok(())
    } else {
        println!("[BinanceAdapter] Private trade execution failed. Code: {}, Response: {}", status, body);
        Err(format!("Binance trade failed: {}", body))
    }
}
