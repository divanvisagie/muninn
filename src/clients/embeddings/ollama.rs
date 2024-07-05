use tracing::{error, info};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;  // Add this dependency in your `Cargo.toml`

use super::EmbeddingsClient;

pub struct OllamaEmbeddingsClient {
    base_url: &'static str,
    model: String,
}

impl OllamaEmbeddingsClient {
    pub fn new(model: &Option<String>) -> Self {
        let model = model.clone();
        OllamaEmbeddingsClient {
            base_url: "http://localhost:11434",
            model: model.unwrap_or("all-minilm".to_string()),
        }
    }
}

#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
}

#[derive(Deserialize)]
struct OllamaResponse {
    embedding: Vec<f32>,
}

/// Benchmark leaderboard: https://huggingface.co/spaces/mteb/leaderboard
pub const OLLAMA_MODELS: [&str; 3] = ["all-minilm", "mxbai-embed-large", "nomic-embed-text"];

async fn get_one(request: OllamaRequest, base_url: &str) -> Result<OllamaResponse> {
    info!("Request: {:?}", request.prompt);
    let url = format!("{}/api/embeddings", base_url);
    let client = reqwest::Client::new();
    let request_body = serde_json::to_string(&request)?;
    let response = client.post(url).body(request_body).send().await?;
    let ollama_response = response.text().await?;
    let response_object: OllamaResponse = serde_json::from_str(&ollama_response)?;
    Ok(response_object)
}


#[async_trait]
impl EmbeddingsClient for OllamaEmbeddingsClient {
    async fn get_embeddings(&self, text: &[&str]) -> Result<Vec<Vec<f32>>> {
        let futs: Vec<_> = text.iter().map(|&t| {
            let request = OllamaRequest {
                model: self.model.to_string(),
                prompt: t.to_string(),
            };
            get_one(request, self.base_url)
        }).collect();

        let responses = futures::future::join_all(futs).await;

        let mut embeddings = Vec::new();

        for response in responses {
            match response {
                Ok(r) => embeddings.push(r.embedding),
                Err(e) => {
                    error!("Error in response object: {}", e);
                    return Err(anyhow::anyhow!("Error in response object"))
                }
            }
        }

        Ok(embeddings)
    }
}
