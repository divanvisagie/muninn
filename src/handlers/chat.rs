use actix_web::{web, HttpResponse};
use uuid::Uuid;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ChatRequest {
    pub role: String,
    pub content: String,
    pub hash: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ChatResponse {
    pub role: String,
    pub content: String,
    pub hash: String,
    pub embedding: Vec<f32>,
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

pub fn save_chat(chat: ChatRequest) -> ChatResponse {
    // Load the pre-trained model
    let chat = ChatResponse {
        role: chat.role,
        content: chat.content,
        hash: chat.hash,
        embedding: vec![0.1, 0.2, 0.3],
    };
    // Stub for embeddings
    return chat;
}

pub fn get_chat(id: String) -> ChatResponse {
    let mut chat = mock_response();
    chat.hash = id;
    return chat;
}
