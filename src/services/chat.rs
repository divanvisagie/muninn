use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::repos::messages::ChatModel;
use std::sync::Arc;
use tokio::sync::Mutex; 

#[derive(Deserialize, Serialize)]
pub struct ChatRequest {
    pub role: String,
    pub content: String,
    pub hash: String,
}

#[derive(Deserialize)]
pub struct SearchRequest {
    pub content: String,
}


#[derive(Deserialize, Serialize, Clone)]
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

#[derive(Deserialize, Serialize, Clone)]
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

#[derive(Clone)]
pub struct ChatService {
    pub(crate) embedding_client: Arc<Mutex<dyn crate::clients::embeddings::EmbeddingsClient>>,
    pub(crate) message_repo: Arc<Mutex<dyn crate::repos::messages::MessageRepo>>,
}

impl ChatService {
    pub async fn get_context(&self, username: &String) -> Result<Vec<ChatResponse>, ()> {
        let chats = self
            .message_repo
            .lock()
            .await
            .get_all_for_user(username.clone());

        //lets cut off at the last 10 messages
        let chats = match chats {
            Ok(chats) => {
                let len = chats.len();
                if len > 50 {
                    chats[len - 10..].to_vec()
                } else {
                    chats
                }
            }
            Err(_) => {
                error!("Failed to get_context");
                return Err(());
            }
        };

        // print the text of the chats
        for chat in chats.iter() {
            info!("{}: {}", chat.role, chat.content);
        }

        Ok(chats
            .iter()
            .map(|chat| ChatResponse::from_model(chat.clone()))
            .collect())
    }

    pub async fn save_chat(
        &self,
        username: &String,
        chat: ChatRequest,
    ) -> Result<ChatResponse, ()> {
        let embeddings_client = self.embedding_client.lock().await;
        let embeddings_result = embeddings_client.get_embeddings(chat.content.clone()).await;

        let embeddings = match embeddings_result {
            Ok(embeddings) => embeddings,
            Err(_) => {
                error!("Failed to get embeddings");
                return Err(());
            }
        };

        let chat_model = ChatModel {
            role: chat.role.clone(),
            content: chat.content.clone(),
            hash: chat.hash.clone(),
            embedding: embeddings.clone(),
            timestamp: chrono::Utc::now().timestamp(),
        };

        let mut message_repo = self.message_repo.lock().await;
        let today = chrono::Utc::now().date_naive();
        let result = message_repo.save_chat(today, username.clone(), chat_model.clone());
        let chat_response = ChatResponse::from_model(result);
        Ok(chat_response)
    }

    pub async fn get_chat(&self, username: &String, id: &String) -> Result<ChatResponse, ()> {
        let chat = match self
            .message_repo
            .lock()
            .await
            .get_chat(username.clone(), id.clone())
        {
            Ok(chat) => chat,
            Err(_) => {
                error!("Failed to get_chat");
                return Err(());
            }
        };

        let chat_response = ChatResponse::from_model(chat);
        Ok(chat_response.clone())
    }

    pub async fn search_chat(
        &self,
        username: &String,
        query: &String,
    ) -> Result<Vec<SearchResponse>, ()> {
        let repo = self.message_repo.lock().await;

        let embeddings_client = self.embedding_client.lock().await;
        let query_vector = embeddings_client
            .get_embeddings(query.clone())
            .await
            .unwrap();

        let founds = repo.embeddings_search_for_user(username.clone(), query_vector);
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
    use std::borrow::Borrow;

    use crate::{clients::embeddings::MockEmbeddingsClient, repos::messages::MessageRepo};

    use super::*;
    use uuid::Uuid;

    struct MockMessageRepo {
        chats: Vec<ChatModel>,
    }

    impl MockMessageRepo {
        fn new() -> MockMessageRepo {
            MockMessageRepo {
                chats: vec![ChatModel {
                    role: "user".to_string(),
                    content: "Hello".to_string(),
                    hash: "123".to_string(),
                    embedding: vec![0.0, 0.0, 0.0],
                    timestamp: chrono::Utc::now().timestamp(),
                }],
            }
        }
    }

    impl MessageRepo for MockMessageRepo {
        fn get_all_for_user_on_day(
            &self,
            _username: String,
            _date: chrono::NaiveDate,
        ) -> Result<Vec<ChatModel>, ()> {
            Ok(self.chats.clone())
        }
        fn save_chat(
            &mut self,
            _date: chrono::NaiveDate,
            _username: String,
            chat: ChatModel,
        ) -> ChatModel {
            self.chats.push(chat.clone());
            chat
        }

        fn get_chat(&mut self, _username: String, id: String) -> Result<ChatModel, ()> {
            let chat = self
                .chats
                .iter()
                .find(|chat| chat.hash == id)
                .unwrap()
                .clone();
            Ok(chat)
        }

        fn get_all_for_user(&self, _username: String) -> Result<Vec<ChatModel>, ()> {
            Ok(self.chats.clone())
        }

        fn embeddings_search_for_user(
            &self,
            _username: String,
            _query_vector: Vec<f32>,
        ) -> Vec<(f32, ChatModel)> {
            let mut result = vec![];
            for chat in self.chats.iter() {
                result.push((0.5, chat.clone()));
            }
            result
        }
    }

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

        let chat_handler = ChatService {
            embedding_client: mock_embeddings.clone(),
            message_repo: mock_repo.clone(),
        };

        chat_handler
            .save_chat("test_user".to_string().borrow(), chat)
            .await
            .unwrap();

        let got_chat = chat_handler
            .get_chat("test_user".to_string().borrow(), &id)
            .await
            .unwrap();

        assert_eq!(got_chat.role, expected_role);
        assert_eq!(got_chat.content, expected_content);
        assert_eq!(got_chat.hash, expected_hash);
    }

    #[tokio::test]
    async fn test_search_chat() {
        let mock_repo = Arc::new(Mutex::new(MockMessageRepo::new()));
        let mock_embeddings = Arc::new(Mutex::new(MockEmbeddingsClient::new()));

        let chat_handler = ChatService {
            embedding_client: mock_embeddings.clone(),
            message_repo: mock_repo.clone(),
        };

        let query = "Hello".to_string();
        let founds = chat_handler
            .search_chat("test_user".to_string().borrow(), &query)
            .await
            .unwrap();
        assert_eq!(founds.len(), 1);
        assert!(founds[0].ranking > 0.0);
    }

    #[tokio::test]
    async fn test_get_context() {
        let mock_repo = Arc::new(Mutex::new(MockMessageRepo::new()));
        let mock_embeddings = Arc::new(Mutex::new(MockEmbeddingsClient::new()));

        let chat_handler = ChatService {
            embedding_client: mock_embeddings.clone(),
            message_repo: mock_repo.clone(),
        };

        let context = chat_handler
            .get_context("test_user".to_string().borrow())
            .await
            .unwrap();
        assert_eq!(context.len(), 1);
    }
}
