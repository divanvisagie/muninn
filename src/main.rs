use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use reqwest::{get, Client};

use crate::handlers::chat::ChatRequest;

mod handlers;
mod repos;
// Lets save chatgpt style chat history at this endpoint
#[post("/v1/chat")]
async fn save_chat(payload: web::Json<ChatRequest>) -> HttpResponse {
    // Load the pre-trained model
    let chat = payload.into_inner();
    let chat = handlers::chat::save_chat(chat);
    HttpResponse::Ok().json(chat)
}

#[get("/v1/chat/{id}")]
async fn get_chat(params: web::Path<(String,)>) -> impl Responder {
    let id = &params.0;
    println!("ID: {}", id);
    let chat = handlers::chat::get_chat(id);
    HttpResponse::Ok().json(chat)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(save_chat).service(get_chat))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
