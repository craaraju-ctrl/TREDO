use super::*;
use tokio::time::{interval, Duration};
use serde::Deserialize;

#[derive(Deserialize)]
struct KucoinSymbolsResponse {
    code: String,
    data: Vec<KucoinSymbol>,
}

#[derive(Deserialize)]
struct KucoinSymbol {
    symbol: String,
    #[serde(rename = "enableTrading")]
    enable_trading: bool,
}

pub struct KucoinAdapter {
    execution_tx: mpsc::Sender<ExecutionCommand>,
    symbol_id_map: Arc<DashMap<String, u16>>,
}

impl KucoinAdapter {
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
        println!("[KucoinAdapter] Starting dynamic token/coin sync...");
        
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("TREDO-Sethu-Exchange-Core/1.0.0")
            .build();

        match client {
            Ok(cli) => {
                match cli.get("https://api.kucoin.com/api/v1/symbols").send().await {
                    Ok(resp) => {
                        if let Ok(res) = resp.json::<KucoinSymbolsResponse>().await {
                            if res.code == "200000" {
                                let mut count = 0;
                                for (idx, sym) in res.data.iter().filter(|s| s.enable_trading).enumerate() {
                                    // Start IDs at offset to prevent overlap with Binance
                                    self.symbol_id_map.insert(sym.symbol.clone(), (idx as u16).wrapping_add(5000));
                                    count += 1;
                                }
                                println!("[KucoinAdapter] Successfully synced {} active tokens/coins from KuCoin Symbols API!", count);
                            } else {
                                println!("[KucoinAdapter] [Warning] KuCoin API returned non-success code: {}. Using default fallback list.", res.code);
                                self.load_defaults();
                            }
                        } else {
                            println!("[KucoinAdapter] [Warning] Failed to deserialize KuCoin symbols response. Using default fallback list.");
                            self.load_defaults();
                        }
                    }
                    Err(e) => {
                        println!("[KucoinAdapter] [Warning] REST call to KuCoin symbols failed: {}. Using default fallback list.", e);
                        self.load_defaults();
                    }
                }
            }
            Err(_) => {
                self.load_defaults();
            }
        }

        let mut interval = interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            let _ = self.execution_tx.send(ExecutionCommand::UpdateBalance("USDT".to_string(), 10000.0)).await;
        }
    }

    fn load_defaults(&self) {
        let defaults = ["BTC-USDT", "ETH-USDT", "SOL-USDT", "KCS-USDT", "ADA-USDT", "XRP-USDT"];
        for (idx, sym) in defaults.iter().enumerate() {
            self.symbol_id_map.insert(sym.to_string(), (idx as u16).wrapping_add(5000));
        }
        println!("[KucoinAdapter] Loaded {} default fallback symbols.", defaults.len());
    }
}

use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;

type HmacSha256 = Hmac<Sha256>;

pub async fn execute_order(symbol: &str, side: &str, amount: f64, price: Option<f64>) -> Result<(), String> {
    let api_key = std::env::var("KUCOIN_API_KEY").unwrap_or_default();
    let secret_key = std::env::var("KUCOIN_SECRET_KEY").unwrap_or_default();
    let passphrase = std::env::var("KUCOIN_PASSPHRASE").unwrap_or_default();
    
    if api_key.is_empty() || secret_key.is_empty() {
        return Err("KuCoin API keys not set in environment.".to_string());
    }

    let client = reqwest::Client::builder()
        .user_agent("TREDO-Exchange-Core/1.0.0")
        .build()
        .map_err(|e| e.to_string())?;

    let timestamp = chrono::Utc::now().timestamp_millis().to_string();
    let method = "POST";
    let endpoint = "/api/v1/orders";
    
    let client_oid = uuid::Uuid::new_v4().to_string();
    let order_type = if price.is_some() { "limit" } else { "market" };
    
    let mut payload = serde_json::json!({
        "clientOid": client_oid,
        "side": side.to_lowercase(),
        "symbol": symbol,
        "type": order_type,
    });
    
    if order_type == "limit" {
        payload["price"] = serde_json::json!(price.expect("Limit orders must have a price").to_string());
        payload["size"] = serde_json::json!(amount.to_string());
    } else {
        payload["size"] = serde_json::json!(amount.to_string());
    }
    
    let body_str = serde_json::to_string(&payload).unwrap_or_default();
    
    // Formula: timestamp + method + endpoint + body
    let sign_str = format!("{}{}{}{}", timestamp, method, endpoint, body_str);
    
    let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes())
        .map_err(|_| "Failed to create HMAC key".to_string())?;
    mac.update(sign_str.as_bytes());
    let signature = BASE64_STANDARD.encode(mac.finalize().into_bytes());

    let mut pass_mac = HmacSha256::new_from_slice(secret_key.as_bytes())
        .map_err(|_| "Failed to create HMAC key".to_string())?;
    pass_mac.update(passphrase.as_bytes());
    let pass_sign = BASE64_STANDARD.encode(pass_mac.finalize().into_bytes());

    println!("[KucoinAdapter] Dispatching signed private trade: POST https://api.kucoin.com/api/v1/orders");
    let resp = client.post("https://api.kucoin.com/api/v1/orders")
        .header("KC-API-KEY", &api_key)
        .header("KC-API-SIGN", &signature)
        .header("KC-API-TIMESTAMP", &timestamp)
        .header("KC-API-PASSPHRASE", &pass_sign)
        .header("KC-API-KEY-VERSION", "2")
        .header("Content-Type", "application/json")
        .body(body_str)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    if status.is_success() {
        println!("[KucoinAdapter] Private trade successfully executed: {}", body);
        Ok(())
    } else {
        println!("[KucoinAdapter] Private trade execution failed. Code: {}, Response: {}", status, body);
        Err(format!("KuCoin trade failed: {}", body))
    }
}
