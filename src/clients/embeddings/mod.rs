use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use self::ollama::OllamaEmbeddingsClient;
use self::fastembed::FastEmbeddingsClient;

pub mod ollama;
pub mod fastembed;

#[derive(Debug, Serialize, Deserialize)]
struct EmbeddingsRequest {
    input: String,
    model: String,
}

#[allow(dead_code)]
pub enum EmbeddingsClientInstance {
    Ollama(OllamaEmbeddingsClient),
    FastEmbed(FastEmbeddingsClient),
}

#[async_trait]
impl EmbeddingsClient for EmbeddingsClientInstance {
    async fn get_embeddings(&self, text: &[&str]) -> Result<Vec<Vec<f32>>> {
        match self {
            EmbeddingsClientInstance::Ollama(client) => client.get_embeddings(text).await,
            EmbeddingsClientInstance::FastEmbed(client) => client.get_embeddings(text).await,
        }
    }
}

#[async_trait]
pub trait EmbeddingsClient: Send + Sync {
    async fn get_embeddings(&self, text: &[&str]) -> Result<Vec<Vec<f32>>>;
}


// create mock type for testing
pub struct MockEmbeddingsClient;

impl MockEmbeddingsClient {
    #[allow(unused)]
    pub fn new() -> Self {
        MockEmbeddingsClient {}
    }
}

#[async_trait]
impl EmbeddingsClient for MockEmbeddingsClient {
    async fn get_embeddings(&self, _text: &[&str]) -> Result<Vec<Vec<f32>>> {
        Ok(vec![vec![0.0, 0.0, 0.0]])
    }
}
