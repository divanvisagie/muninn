use uuid::Uuid;

use crate::handlers::chat::{ChatRequest, ChatResponse};

#[derive(Clone)]
pub struct ChatModel {
    pub role: String,
    pub content: String,
    pub hash: String,
    pub embedding: Vec<f32>,
}

pub struct FsMessageRepo {
    memory: std::collections::HashMap<(String, String), ChatModel>, // Update HashMap key to include user
}

pub trait MessageRepo: Send + Sync {
    fn save_chat(&mut self, user: String, chat: ChatModel) -> ChatModel;
    fn get_chat(&self, user: String, id: String) -> ChatModel; // Add user parameter
    fn embeddings_search_for_user(&self, user: String, query_vector: Vec<f32>) -> Vec<ChatModel>;
}

impl FsMessageRepo {
    pub fn new() -> FsMessageRepo {
        FsMessageRepo {
            memory: std::collections::HashMap::new(),
        }
    }

    fn get_all_for_user(&self, user: String) -> Vec<ChatModel> {
        let mut chats = vec![];
        for ((_, u), chat) in &self.memory {
            if u == &user {
                chats.push(chat.clone());
            }
        }
        return chats;
    }
}

fn cosine_similarity(v1: &Vec<f32>, v2: &Vec<f32>) -> f32 {
    let dot_product = v1.iter().zip(v2).map(|(a, b)| a * b).sum::<f32>();
    let magnitude_v1 = (v1.iter().map(|a| a.powi(2)).sum::<f32>()).sqrt();
    let magnitude_v2 = (v2.iter().map(|a| a.powi(2)).sum::<f32>()).sqrt();
    let magnitude_product = magnitude_v1 * magnitude_v2;
    dot_product / magnitude_product
}

impl MessageRepo for FsMessageRepo {
    fn save_chat(&mut self, user: String, chat: ChatModel) -> ChatModel {
        let key = (chat.hash.clone(), user); // Create key using hash and user
        self.memory.insert(key, chat.clone());
        // Stub for embeddings
        return chat;
    }

    fn get_chat(&self, user: String, id: String) -> ChatModel {
        let key = (id, user); // Create key using id and user
        let chat = self.memory.get(&key).unwrap();
        return chat.clone();
    }

    fn embeddings_search_for_user(&self, user: String, query_vector: Vec<f32>) -> Vec<ChatModel> {
        let chats = self.get_all_for_user(user);

        // rank the chats by similarity to the query_vector
        let mut ranked_chats: Vec<(f32, ChatModel)> = vec![];
        for chat in chats {
            let similarity = cosine_similarity(&chat.embedding, &query_vector);
            ranked_chats.push((similarity, chat));
        }

        //convert to ordered vector and return Vec<ChatModel>
        ranked_chats.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        let ranked_chats: Vec<ChatModel> =
            ranked_chats.iter().map(|(_, chat)| chat.clone()).collect();
        return ranked_chats;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_save_chat() {
        let chat = ChatModel {
            role: "user".to_string(),
            content: "Hello".to_string(),
            hash: Uuid::new_v4().to_string(),
            embedding: vec![0.1, 0.2, 0.3],
        };
        let id = chat.hash.clone();
        let expected_hash = id.clone();
        let expected_role = chat.role.clone();
        let expected_content = chat.content.clone();

        let mut repo = FsMessageRepo::new();
        repo.save_chat("test-user".to_string(), chat.clone()); // Pass user parameter

        let got_chat = repo.get_chat(id, "test-user".to_string()); // Pass user parameter
        assert_eq!(got_chat.role, expected_role);
        assert_eq!(got_chat.content, expected_content);
        assert_eq!(got_chat.hash, expected_hash);
    }
}
