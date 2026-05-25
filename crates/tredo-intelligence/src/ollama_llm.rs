use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use std::time::Duration;

use tredo_core::{LLMProvider, LLMParams, ProviderError};

/// Ollama LLM Provider — plugs into the PluginRegistry as a replaceable local LLM backend.
///
/// Implements complete, stream_complete, and embed using local Ollama REST endpoints.
#[derive(Debug)]
pub struct OllamaLLM {
    endpoint: String,
    model: String,
    client: reqwest::Client,
}

impl OllamaLLM {
    /// Create a new OllamaLLM provider.
    ///
    /// `endpoint` defaults to `"http://localhost:11434"`.
    /// `model` defaults to `"nemetron:4b"` or `OLLAMA_MODEL` env var.
    pub fn new(endpoint: Option<String>, model: Option<String>) -> Self {
        let endpoint = endpoint.unwrap_or_else(|| "http://localhost:11434".to_string());
        let model = model
            .or_else(|| std::env::var("OLLAMA_MODEL").ok())
            .unwrap_or_else(|| "nemetron:4b".to_string());

        Self {
            endpoint,
            model,
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to build HTTP client for Ollama"),
        }
    }
}

#[async_trait]
impl LLMProvider for OllamaLLM {
    fn provider_name(&self) -> &str {
        "ollama"
    }

    fn model_name(&self) -> &str {
        &self.model
    }

    async fn complete(
        &self,
        prompt: &str,
        system: Option<&str>,
        params: Option<LLMParams>,
    ) -> Result<String, ProviderError> {
        let url = format!("{}/api/generate", self.endpoint);
        
        let temperature = params.as_ref().map(|p| p.temperature).unwrap_or(0.2);
        
        let mut req_body = serde_json::json!({
            "model": self.model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": temperature
            }
        });

        if let Some(sys) = system {
            req_body["system"] = serde_json::json!(sys);
        }

        let resp = self
            .client
            .post(&url)
            .json(&req_body)
            .send()
            .await
            .map_err(|e| ProviderError::ConnectionError(format!("Ollama request failed: {}", e)))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(ProviderError::ExternalError(format!(
                "Ollama returned status {}: {}",
                status, body
            )));
        }

        let ollama_resp: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ProviderError::AnalysisError(format!("Failed to parse Ollama JSON: {}", e)))?;

        let response_text = ollama_resp["response"]
            .as_str()
            .ok_or_else(|| ProviderError::AnalysisError("Ollama response field is missing or empty".to_string()))?;

        Ok(response_text.to_string())
    }

    async fn stream_complete(
        &self,
        prompt: &str,
        system: Option<&str>,
        params: Option<LLMParams>,
    ) -> Result<BoxStream<'static, String>, ProviderError> {
        let url = format!("{}/api/generate", self.endpoint);
        
        let temperature = params.as_ref().map(|p| p.temperature).unwrap_or(0.2);
        
        let mut req_body = serde_json::json!({
            "model": self.model,
            "prompt": prompt,
            "stream": true,
            "options": {
                "temperature": temperature
            }
        });

        if let Some(sys) = system {
            req_body["system"] = serde_json::json!(sys);
        }

        let client = self.client.clone();

        let stream = async_stream::stream! {
            let resp = match client.post(&url).json(&req_body).send().await {
                Ok(r) => r,
                Err(e) => {
                    yield format!("Error: {}", e);
                    return;
                }
            };

            let mut stream = resp.bytes_stream();
            let mut buffer = Vec::new();

            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(bytes) => {
                        buffer.extend_from_slice(&bytes);
                        
                        // Parse line by line
                        while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
                            let line_bytes = buffer.drain(..=pos).collect::<Vec<u8>>();
                            if let Ok(line_str) = String::from_utf8(line_bytes) {
                                let trimmed = line_str.trim();
                                if !trimmed.is_empty() {
                                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
                                        if let Some(response_chunk) = val["response"].as_str() {
                                            yield response_chunk.to_string();
                                        }
                                        if val["done"].as_bool().unwrap_or(false) {
                                            return;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        yield format!("Error: {}", e);
                        return;
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }

    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f64>>, ProviderError> {
        let url = format!("{}/api/embed", self.endpoint);
        let mut embeddings = Vec::new();

        for text in texts {
            let req_body = serde_json::json!({
                "model": self.model,
                "input": text
            });

            let resp = self
                .client
                .post(&url)
                .json(&req_body)
                .send()
                .await
                .map_err(|e| ProviderError::ConnectionError(format!("Ollama embedding request failed: {}", e)))?;

            let status = resp.status();
            if !status.is_success() {
                let body = resp.text().await.unwrap_or_default();
                return Err(ProviderError::ExternalError(format!(
                    "Ollama embedding returned status {}: {}",
                    status, body
                )));
            }

            let embedding_resp: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| ProviderError::AnalysisError(format!("Failed to parse Ollama embedding response: {}", e)))?;

            // Ollama embed response contains a "embeddings" field which is a 2D array if list input, or "embedding" if singular.
            // Let's support both.
            if let Some(arr) = embedding_resp["embeddings"].as_array() {
                if let Some(first) = arr.first() {
                    if let Some(vec) = first.as_array() {
                        let f64_vec: Vec<f64> = vec.iter().filter_map(|v| v.as_f64()).collect();
                        embeddings.push(f64_vec);
                        continue;
                    }
                }
            }
            if let Some(vec) = embedding_resp["embedding"].as_array() {
                let f64_vec: Vec<f64> = vec.iter().filter_map(|v| v.as_f64()).collect();
                embeddings.push(f64_vec);
                continue;
            }

            return Err(ProviderError::AnalysisError("Failed to extract embedding from Ollama response".to_string()));
        }

        Ok(embeddings)
    }
}
