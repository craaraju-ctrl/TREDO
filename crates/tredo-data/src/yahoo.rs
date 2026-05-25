use crate::{MarketDataProvider, TimeFrame};
use tredo_skills::Candle;

/// Free Yahoo Finance market data provider (no API key required)
pub struct YahooFinanceProvider {
    client: reqwest::Client,
}

impl YahooFinanceProvider {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .user_agent("TREDO-Trading-Terminal/1.0.0 (Mozilla/5.0 compatible)")
            .build()
            .unwrap_or_default();
        Self { client }
    }

    /// Format symbol for Yahoo Finance (e.g., "BTC-USD", "AAPL", "RELIANCE.NS", "GC=F")
    fn format_symbol(symbol: &str) -> String {
        let upper = symbol.to_uppercase().trim().to_string();
        
        // 1. Indian Stocks (e.g., NSE:RELIANCE -> RELIANCE.NS)
        if upper.starts_with("NSE:") {
            if let Some(ticker) = upper.strip_prefix("NSE:") {
                return format!("{}.NS", ticker);
            }
        }
        
        // 2. Commodities mapping
        if upper == "XAU-USD" || upper == "GOLD" || upper.starts_with("XAU") {
            return "GC=F".to_string(); // Gold Futures
        }
        if upper == "XAG-USD" || upper == "SILVER" || upper.starts_with("XAG") {
            return "SI=F".to_string(); // Silver Futures
        }
        if upper == "USOIL" || upper == "WTI" {
            return "CL=F".to_string(); // Crude Oil Futures
        }
        if upper == "NGAS" || upper == "NATGAS" {
            return "NG=F".to_string(); // Natural Gas Futures
        }
        
        // 3. Fallback standard replacement
        upper.replace('/', "-")
    }

    fn parse_yahoo_response(&self, json: &serde_json::Value) -> Result<Vec<Candle>, String> {
        let result = json["chart"]["result"]
            .as_array()
            .and_then(|r| r.first())
            .ok_or_else(|| "No chart result in Yahoo response".to_string())?;

        let timestamps = result["timestamp"]
            .as_array()
            .ok_or_else(|| "No timestamps in Yahoo response".to_string())?;

        let quote = &result["indicators"]["quote"]
            .as_array()
            .and_then(|q| q.first())
            .ok_or_else(|| "No quote data in Yahoo response".to_string())?;

        let opens = quote["open"].as_array().ok_or_else(|| "No open data".to_string())?;
        let highs = quote["high"].as_array().ok_or_else(|| "No high data".to_string())?;
        let lows = quote["low"].as_array().ok_or_else(|| "No low data".to_string())?;
        let closes = quote["close"].as_array().ok_or_else(|| "No close data".to_string())?;
        let volumes = quote["volume"].as_array().ok_or_else(|| "No volume data".to_string())?;

        let mut candles = Vec::with_capacity(timestamps.len());

        for i in 0..timestamps.len() {
            let time = timestamps[i].as_i64().unwrap_or(0) * 1000; // Convert to ms
            let open = opens[i].as_f64().unwrap_or(0.0);
            let high = highs[i].as_f64().unwrap_or(0.0);
            let low = lows[i].as_f64().unwrap_or(0.0);
            let close = closes[i].as_f64().unwrap_or(0.0);
            let volume = volumes[i].as_f64().unwrap_or(0.0);

            // Skip null candles (Yahoo sometimes includes null entries at beginning)
            if open == 0.0 && close == 0.0 {
                continue;
            }

            candles.push(Candle {
                time,
                open,
                high,
                low,
                close,
                volume,
            });
        }

        if candles.is_empty() {
            return Err("No valid candles parsed from Yahoo response".to_string());
        }

        Ok(candles)
    }
}

impl Default for YahooFinanceProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl MarketDataProvider for YahooFinanceProvider {
    async fn fetch_candles(&self, symbol: &str, timeframe: TimeFrame) -> Result<Vec<Candle>, String> {
        let formatted = Self::format_symbol(symbol);
        let interval = timeframe.as_yahoo_interval();
        let range = timeframe.as_yahoo_range();

        let url = format!(
            "https://query1.finance.yahoo.com/v8/finance/chart/{}?interval={}&range={}",
            formatted, interval, range
        );

        println!("[YahooFinance] Fetching {} {} (range: {})", formatted, interval, range);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!(
                "Yahoo Finance returned status {} for symbol {}",
                response.status(),
                formatted
            ));
        }

        let text = response.text().await.map_err(|e| format!("Failed to read response: {}", e))?;
        let json: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| format!("Failed to parse JSON: {}", e))?;

        // Check for error responses
        if let Some(error) = json["chart"]["error"].as_object() {
            if let Some(msg) = error.get("description").and_then(|d| d.as_str()) {
                return Err(format!("Yahoo Finance error: {}", msg));
            }
        }

        self.parse_yahoo_response(&json)
    }

    async fn fetch_current_price(&self, symbol: &str) -> Result<f64, String> {
        let candles = self.fetch_candles(symbol, TimeFrame::Min5).await?;
        candles
            .last()
            .map(|c| c.close)
            .ok_or_else(|| "No price data available".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_symbol() {
        assert_eq!(YahooFinanceProvider::format_symbol("BTC-USD"), "BTC-USD");
        assert_eq!(YahooFinanceProvider::format_symbol("BTC/USD"), "BTC-USD");
        assert_eq!(YahooFinanceProvider::format_symbol("AAPL"), "AAPL");
        assert_eq!(YahooFinanceProvider::format_symbol("NSE:RELIANCE"), "RELIANCE.NS");
        assert_eq!(YahooFinanceProvider::format_symbol("XAU-USD"), "GC=F");
        assert_eq!(YahooFinanceProvider::format_symbol("XAG-USD"), "SI=F");
        assert_eq!(YahooFinanceProvider::format_symbol("USOIL"), "CL=F");
        assert_eq!(YahooFinanceProvider::format_symbol("NGAS"), "NG=F");
    }
}
