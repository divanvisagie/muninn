use axum::{extract::Json, extract::State};
use serde::Deserialize;

use crate::{layers::{security::SecurityLayer, selector::SelectorLayer, Layer}, Resources};


#[derive(Deserialize)]
#[allow(dead_code)]
pub struct FileMessage {
    pub file_type: String,
    pub data: Vec<u8>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct RequestMessage {
    pub chat_id: i64,
    pub telegram_user_id: Option<i64>,
    pub telegram_username: Option<String>,
    pub alternative_usernames: Option<Vec<String>>,
    pub text: String,
    pub files: Option<Vec<FileMessage>>,
}

#[allow(dead_code)]
impl RequestMessage {
    pub fn new(
        chat_id: i64,
        text: String,
    ) -> Self {
        RequestMessage {
            chat_id,
            telegram_user_id: None,
            telegram_username: None,
            alternative_usernames: None,
            text,
            files: None,
        }
    }
}

pub async fn handle_request_message(
    State(_resources): State<Resources>,
    Json(payload): Json<RequestMessage>,
) -> String {
    
    let selecto_layer = SelectorLayer::new();
    let mut security_layer = SecurityLayer::new(Box::new(selecto_layer));

    match security_layer.execute(&payload).await {
        Ok(_) => println!("Security checks passed"),
        Err(e) => println!("Security checks failed: {}", e),
    }

    let name = payload.text.clone();
    format!("Hello, world!, you sent {name}!").to_string()
}
