use uuid::Uuid;

use crate::handlers::chat::{ChatRequest, ChatResponse};

pub struct FsMessageRepo {
    memory: std::collections::HashMap<String, ChatResponse>,
}
pub trait MessageRepo {
    fn save_chat(&mut self, chat: ChatRequest) -> ChatResponse;
    fn get_chat(&self, id: String) -> ChatResponse;
}
impl FsMessageRepo {
    pub fn new() -> FsMessageRepo {
        FsMessageRepo {
            memory: std::collections::HashMap::new(),
        }
    }
}
impl MessageRepo for FsMessageRepo {
    fn save_chat(&mut self, chat: ChatRequest) -> ChatResponse {
        // Load the pre-trained model
        let chat = ChatResponse {
            role: chat.role,
            content: chat.content,
            hash: chat.hash,
            embedding: vec![0.1, 0.2, 0.3],
        };
        self.memory.insert(chat.hash.clone(), chat.clone());
        // Stub for embeddings
        return chat;
    }

    fn get_chat(&self, _id: String) -> ChatResponse {
        let chat = self.memory.get(&_id).unwrap();
        return chat.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_save_chat() {
        let chat = ChatRequest {
            role: "user".to_string(),
            content: "Hello".to_string(),
            hash: Uuid::new_v4().to_string(),
        };
        let id = chat.hash.clone();
        let expected_hash = id.clone();
        let expected_role = chat.role.clone();
        let expected_content = chat.content.clone();

        let mut repo = FsMessageRepo::new();
        repo.save_chat(chat);

        let got_chat = repo.get_chat(id);
        assert_eq!(got_chat.role, expected_role);
        assert_eq!(got_chat.content, expected_content);
        assert_eq!(got_chat.hash, expected_hash);
    }
}
