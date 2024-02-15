use actix_web::{web, HttpResponse};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Chat {
    pub message: String,
}

pub fn save_chat(chat: Chat) -> Chat {
    // Load the pre-trained model

    // Stub for embeddings
    return chat;
}
