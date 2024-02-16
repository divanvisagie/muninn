use actix_web::{http::header::CROSS_ORIGIN_EMBEDDER_POLICY, web, HttpResponse};
use async_trait::async_trait;
use uuid::Uuid;

use crate::{clients::embeddings, repos::messages::ChatModel};
use std::{result, sync::Arc};
use tokio::sync::Mutex; // Import the TryFutureExt trait

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ChatRequest {
    pub role: String,
    pub content: String,
    pub hash: String,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct ChatResponse {
    pub role: String,
    pub content: String,
    pub hash: String,
    pub embedding: Vec<f32>,
}

impl ChatResponse {
    pub fn new(role: String, content: String, hash: String, embedding: Vec<f32>) -> ChatResponse {
        ChatResponse {
            role,
            content,
            hash,
            embedding,
        }
    }
    pub fn from_model(model: ChatModel) -> ChatResponse {
        ChatResponse {
            role: model.role,
            content: model.content,
            hash: model.hash,
            embedding: model.embedding,
        }
    }
}

fn mock_response() -> ChatResponse {
    let chat = ChatResponse {
        role: "user".to_string(),
        content: "Hello".to_string(),
        hash: Uuid::new_v4().to_string(),
        embedding: vec![0.1, 0.2, 0.3],
    };
    return chat;
}

#[derive(Clone)]
pub struct ChatHandlerImpl {
    pub(crate) embedding_client: Arc<Mutex<dyn crate::clients::embeddings::EmbeddingsClient>>,
    pub(crate) message_repo: Arc<Mutex<dyn crate::repos::messages::MessageRepo>>,
}

#[async_trait]
pub trait ChatHandler: Send + Sync {
    async fn save_chat(&self, chat: ChatRequest) -> Result<ChatResponse, ()>;
    async fn get_chat(&self, id: &String) -> Result<ChatResponse, ()>;
}

#[async_trait]
impl ChatHandler for ChatHandlerImpl {
    async fn save_chat(&self, chat: ChatRequest) -> Result<ChatResponse, ()> {
        let embeddings_client = self.embedding_client.lock().await;
        let embeddings_result = embeddings_client.get_embeddings(chat.content.clone()).await;

        let embeddings = match embeddings_result {
            Ok(embeddings) => embeddings,
            Err(_) => return Err(()),
        };

        let cm = ChatModel {
            role: chat.role.clone(),
            content: chat.content.clone(),
            hash: chat.hash.clone(),
            embedding: embeddings.clone(),
        };

        let mut message_repo = self.message_repo.lock().await;
        let result = message_repo.save_chat("my_user".to_string(), cm.clone());
        let cr = ChatResponse::from_model(result);
        Ok(cr)
    }

    async fn get_chat(&self, id: &String) -> Result<ChatResponse, ()> {
        let chat = self
            .message_repo
            .lock()
            .await
            .get_chat("my_user".to_string(), id.clone())
            .unwrap();
        let cr = ChatResponse {
            role: chat.role,
            content: chat.content,
            hash: chat.hash,
            embedding: chat.embedding,
        };
        Ok(cr.clone())
    }
}
