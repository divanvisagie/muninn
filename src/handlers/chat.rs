use actix_web::{web, HttpResponse};
use tracing::error;

use crate::{
    services::chat::{ChatRequest, ChatService, SearchRequest},
    Resources,
};

pub async fn get_chat(
    resources: web::Data<Resources>,
    params: web::Path<(String, String)>,
) -> HttpResponse {
    let resources = resources.into_inner();
    let chat_service = ChatService {
        embedding_client: resources.embeddings_client.clone(),
        message_repo: resources.message_repo.clone(),
    };
    let username = &params.0.clone();
    let id = &params.1.clone();
    let chat = chat_service.get_chat(username, id).await;
    let chat = match chat {
        Ok(chat) => chat,
        Err(_) => {
            error!("Error getting chat");
            return HttpResponse::InternalServerError().finish();
        }
    };
    HttpResponse::Ok().json(chat)
}

pub async fn search_chat(
    resources: web::Data<Resources>,
    params: web::Path<(String,)>,
    payload: web::Json<SearchRequest>,
) -> HttpResponse {
    let resources = resources.into_inner();
    let chat_service = ChatService {
        embedding_client: resources.embeddings_client.clone(),
        message_repo: resources.message_repo.clone(),
    };
    let username = &params.0.clone();
    let query = &payload.content.clone();
    let chat = chat_service.search_chat(username, query).await;

    let chat = match chat {
        Ok(chat) => chat,
        Err(_) => {
            error!("Error searching chat");
            return HttpResponse::InternalServerError().finish();
        }
    };
    HttpResponse::Ok().json(chat)
}


pub async fn get_context_with(
    resources: web::Data<Resources>,
    params: web::Path<(String,)>,
    payload: web::Json<ChatRequest>,
) -> HttpResponse {
    let resources = resources.into_inner();
    let chat_service = ChatService {
        embedding_client: resources.embeddings_client.clone(),
        message_repo: resources.message_repo.clone(),
    };
    let username = &params.0.clone();
    let chat_request = payload.into_inner();
    let chat = chat_service.get_context(username, &chat_request.content).await;

    let chat = match chat {
        Ok(chat) => chat,
        Err(_) => {
            error!("Error getting chat context");
            return HttpResponse::InternalServerError().finish();
        }
    };
    HttpResponse::Ok().json(chat)
}

pub async fn save_chat(
    resources: web::Data<Resources>,
    params: web::Path<(String,)>,
    payload: web::Json<ChatRequest>,
) -> HttpResponse {
    let username = &params.0.clone();
    let resources = resources.into_inner();
    let chat_service = ChatService {
        embedding_client: resources.embeddings_client.clone(),
        message_repo: resources.message_repo.clone(),
    };
    let chat = payload.into_inner();
    let chat = chat_service.save_chat(username, chat).await;

    match chat {
        Ok(chat) => HttpResponse::Ok().json(chat),
        Err(_) => {
            error!("Error saving chat");
            HttpResponse::InternalServerError().finish()
        }
    }
}
