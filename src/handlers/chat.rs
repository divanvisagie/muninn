use async_trait::async_trait;
use tracing::error;

use crate::repos::messages::ChatModel;
use std::sync::Arc;
use tokio::sync::Mutex; // Import the TryFutureExt trait

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ChatRequest {
    pub role: String,
    pub content: String,
    pub hash: String,
}

#[derive(serde::Deserialize)]
pub struct SearchRequest {
    pub content: String,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct SearchResponse {
    pub role: String,
    pub content: String,
    pub hash: String,
    pub ranking: f32,
}
impl SearchResponse {
    fn from_chat_model(clone: ChatModel, ranking: f32) -> SearchResponse {
        SearchResponse {
            role: clone.role,
            content: clone.content,
            hash: clone.hash,
            ranking,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct ChatResponse {
    pub role: String,
    pub content: String,
    pub hash: String,
}

impl ChatResponse {
    #[allow(dead_code)]
    pub fn new(role: String, content: String, hash: String) -> ChatResponse {
        ChatResponse {
            role,
            content,
            hash,
        }
    }
    pub fn from_model(model: ChatModel) -> ChatResponse {
        ChatResponse {
            role: model.role,
            content: model.content,
            hash: model.hash,
        }
    }
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
    async fn search_chat(&self, query: &String) -> Result<Vec<SearchResponse>, ()>;
}

#[async_trait]
impl ChatHandler for ChatHandlerImpl {
    async fn save_chat(&self, chat: ChatRequest) -> Result<ChatResponse, ()> {
        let embeddings_client = self.embedding_client.lock().await;
        let embeddings_result = embeddings_client.get_embeddings(chat.content.clone()).await;

        let embeddings = match embeddings_result {
            Ok(embeddings) => embeddings,
            Err(_) => {
                error!("Failed to get embeddings");
                return Err(());
            }
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
        let chat = match self
            .message_repo
            .lock()
            .await
            .get_chat("my_user".to_string(), id.clone())
        {
            Ok(chat) => chat,
            Err(_) => {
                error!("Failed to get chat");
                return Err(());
            }
        };

        let cr = ChatResponse::from_model(chat);
        Ok(cr.clone())
    }

    async fn search_chat(&self, query: &String) -> Result<Vec<SearchResponse>, ()> {
        let repo = self.message_repo.lock().await;
        let user = "my_user".to_string();

        let embeddings_client = self.embedding_client.lock().await;
        let query_vector = embeddings_client
            .get_embeddings(query.clone())
            .await
            .unwrap();

        let founds = repo.embeddings_search_for_user(user, query_vector);
        let founds = founds
            .iter()
            .map(|(similarity, chat)| {
                SearchResponse::from_chat_model(chat.clone(), similarity.clone())
            })
            .collect();
        Ok(founds)
    }
}

#[cfg(test)]
mod tests {
    use crate::{clients::embeddings::MockEmbeddingsClient, repos::messages::MockMessageRepo};

    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_save_chat_and_get_chat() {
        let id = Uuid::new_v4().to_string();
        let chat = ChatRequest {
            role: "user".to_string(),
            content: "Hello".to_string(),
            hash: id.clone(),
        };
        let expected_hash = id.clone();
        let expected_role = chat.role.clone();
        let expected_content = chat.content.clone();

        let mock_repo = Arc::new(Mutex::new(MockMessageRepo::new()));
        let mock_embeddings = Arc::new(Mutex::new(MockEmbeddingsClient::new()));

        let chat_handler = ChatHandlerImpl {
            embedding_client: mock_embeddings.clone(),
            message_repo: mock_repo.clone(),
        };

        chat_handler.save_chat(chat).await.unwrap();

        let got_chat = chat_handler.get_chat(&id).await.unwrap();
        assert_eq!(got_chat.role, expected_role);
        assert_eq!(got_chat.content, expected_content);
        assert_eq!(got_chat.hash, expected_hash);
    }

    #[tokio::test]
    async fn test_search_chat() {
        let mock_repo = Arc::new(Mutex::new(MockMessageRepo::new()));
        let mock_embeddings = Arc::new(Mutex::new(MockEmbeddingsClient::new()));

        let chat_handler = ChatHandlerImpl {
            embedding_client: mock_embeddings.clone(),
            message_repo: mock_repo.clone(),
        };

        let query = "Hello".to_string();
        let founds = chat_handler.search_chat(&query).await.unwrap();
        assert_eq!(founds.len(), 1);
        assert!(founds[0].ranking > 0.0);
    }
}
