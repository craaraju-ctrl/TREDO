use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use std::time::Duration;

use arkm_core::{LLMProvider, LLMParams, ProviderError};

/// Gemini LLM Provider — plugs into the PluginRegistry as a replaceable LLM backend.
///
/// Extracted from the old IntelligencePool to be a standalone provider.
/// Uses the Gemini REST API directly via reqwest.
#[derive(Debug)]
pub struct GeminiLLM {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl GeminiLLM {
    /// Create a new GeminiLLM provider.
    ///
    /// `model` defaults to `"gemini-2.5-flash"`. `api_key` defaults to `GEMINI_API_KEY` env var.
    pub fn new(api_key: Option<String>, model: Option<String>) -> Self {
        let api_key = api_key
            .or_else(|| std::env::var("GEMINI_API_KEY").ok())
            .expect("GEMINI_API_KEY must be set via env var or passed explicitly");

        Self {
            api_key,
            model: model.unwrap_or_else(|| "gemini-2.5-flash".to_string()),
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(15))
                .build()
                .expect("Failed to build HTTP client"),
        }
    }

    /// Build the Gemini API URL for content generation
    fn api_url(&self) -> String {
        format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        )
    }
}

#[async_trait]
impl LLMProvider for GeminiLLM {
    fn provider_name(&self) -> &str {
        "gemini"
    }

    fn model_name(&self) -> &str {
        &self.model
    }

    async fn complete(
        &self,
        prompt: &str,
        _system: Option<&str>,
        _params: Option<LLMParams>,
    ) -> Result<String, ProviderError> {
        // Condensed prompt for KV cache efficiency
        let condensed = format!(
            "HFT analyst. Query: \"{}\". Reply JSON only: {{\"conviction\":0.0-1.0,\"reasoning\":\"max 2 sentences\"}}",
            prompt
        );

        let req_body = serde_json::json!({
            "contents": [{
                "parts": [{"text": condensed}]
            }],
            "generationConfig": {
                "responseMimeType": "application/json"
            }
        });

        let resp = self
            .client
            .post(self.api_url())
            .json(&req_body)
            .send()
            .await
            .map_err(|e| ProviderError::ConnectionError(format!("Gemini API request failed: {}", e)))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(ProviderError::ExternalError(format!(
                "Gemini API returned {}: {}",
                status, body
            )));
        }

        let gemini_resp: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ProviderError::AnalysisError(format!("Failed to parse Gemini response: {}", e)))?;

        // Extract text from response
        let text = gemini_resp["candidates"]
            .as_array()
            .and_then(|c| c.first())
            .and_then(|c| c["content"]["parts"].as_array())
            .and_then(|p| p.first())
            .and_then(|p| p["text"].as_str())
            .ok_or_else(|| ProviderError::AnalysisError("Gemini returned empty response".to_string()))?;

        Ok(text.to_string())
    }

    async fn stream_complete(
        &self,
        prompt: &str,
        _system: Option<&str>,
        _params: Option<LLMParams>,
    ) -> Result<BoxStream<'static, String>, ProviderError> {
        // For streaming, use the streamGenerateContent endpoint
        let stream_url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?alt=sse&key={}",
            self.model, self.api_key
        );

        let req_body = serde_json::json!({
            "contents": [{
                "parts": [{"text": prompt}]
            }],
            "generationConfig": {
                "responseMimeType": "application/json"
            }
        });

        let client = self.client.clone();

        let stream = async_stream::stream! {
            let resp = match client.post(&stream_url).json(&req_body).send().await {
                Ok(r) => r,
                Err(e) => {
                    yield format!("Error: {}", e);
                    return;
                }
            };

            let mut stream = resp.bytes_stream();
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(bytes) => {
                        if let Ok(text) = String::from_utf8(bytes.to_vec()) {
                            // Parse SSE format: "data: {...}\n\n"
                            for line in text.lines() {
                                if let Some(data) = line.strip_prefix("data: ") {
                                    if data == "[DONE]" { return; }
                                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(data) {
                                        if let Some(text) = val["candidates"]
                                            .as_array()
                                            .and_then(|c| c.first())
                                            .and_then(|c| c["content"]["parts"].as_array())
                                            .and_then(|p| p.first())
                                            .and_then(|p| p["text"].as_str())
                                        {
                                            yield text.to_string();
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        yield format!("Error: {}", e);
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }

    async fn embed(&self, _texts: &[String]) -> Result<Vec<Vec<f64>>, ProviderError> {
        // Gemini embedding API is separate; for now return empty
        Err(ProviderError::NotImplemented(
            "Gemini embedding via API not yet implemented".to_string(),
        ))
    }
}
