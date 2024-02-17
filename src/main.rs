use std::sync::Arc;

use actix_web::{web, App, HttpResponse, HttpServer};
use clients::embeddings::OpenAiEmbeddingsClient;
use handlers::chat::{ChatHandler, ChatHandlerImpl, SearchRequest};
use repos::messages::FsMessageRepo;
use tokio::sync::Mutex;
use tracing::error;

use crate::handlers::chat::ChatRequest;

mod clients;
mod handlers;
mod repos;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    let chat_handler = ChatHandlerImpl {
        embedding_client: Arc::new(Mutex::new(OpenAiEmbeddingsClient::new())),
        message_repo: Arc::new(Mutex::new(FsMessageRepo::new())),
    };

    let data = web::Data::new(chat_handler);

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .route("/api/v1/chat", web::post().to(save_chat))
            .route("/api/v1/chat/{id}", web::get().to(get_chat))
            .route("/api/v1/chat/search", web::post().to(search_chat))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

async fn save_chat(
    chat_handler: web::Data<ChatHandlerImpl>,
    payload: web::Json<ChatRequest>,
) -> HttpResponse {
    let chat_handler = chat_handler.into_inner();
    let chat = payload.into_inner();
    let chat = chat_handler.save_chat(chat).await;

    //Check the result and return the appropriate response
    match chat {
        Ok(chat) => HttpResponse::Ok().json(chat),
        Err(_) => {
            error!("Error saving chat");
            HttpResponse::InternalServerError().finish()
        }
    }
}

async fn get_chat(
    chat_handler: web::Data<ChatHandlerImpl>,
    params: web::Path<(String,)>,
) -> HttpResponse {
    let chat_handler = chat_handler.into_inner();
    let id = &params.0.clone();
    println!("ID: {}", id);
    let chat = chat_handler.get_chat(id).await;
    let chat = match chat {
        Ok(chat) => chat,
        Err(_) => {
            error!("Error getting chat");
            return HttpResponse::InternalServerError().finish();
        }
    };
    HttpResponse::Ok().json(chat)
}

async fn search_chat(
    chat_handler: web::Data<ChatHandlerImpl>,
    payload: web::Json<SearchRequest>,
) -> HttpResponse {
    let chat_handler = chat_handler.into_inner();
    let query = &payload.content.clone();
    let chat = chat_handler.search_chat(query).await;
    let chat = match chat {
        Ok(chat) => chat,
        Err(_) => {
            error!("Error searching chat");
            return HttpResponse::InternalServerError().finish();
        }
    };
    HttpResponse::Ok().json(chat)
}
