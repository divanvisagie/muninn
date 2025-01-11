use actix_web::{web, HttpResponse};
use serde::Deserialize;

use crate::Resources;

#[derive(Deserialize)]
pub struct FileMessage {
    pub fileType: String,
    pub data: Vec<u8>,
}

#[derive(Deserialize)]
pub struct RequestMessage {
    pub chatId: i64,
    pub telegramUserId: Option<i64>,
    pub telegramUsername: Option<String>,
    pub alternativeUsernames: Option<Vec<String>>,
    pub text: String,
    pub files: Option<Vec<FileMessage>>,
}

pub async fn handle_request_message(
    resources: web::Data<Resources>,
    payload: web::Json<RequestMessage>,
) -> HttpResponse {
    HttpResponse::Ok().json("Hello, world!")
}
