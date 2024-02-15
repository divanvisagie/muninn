use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use reqwest::Client;
use serde_json::json;

#[derive(serde::Deserialize, serde::Serialize)]
struct Chat {
    pub message: String,
}

#[derive(serde::Deserialize)]
struct Payload {
    sentence: String,
}

#[post("/embeddings")]
async fn embeddings(payload: web::Json<Payload>) -> HttpResponse {
    // Load the pre-trained model

    // Stub for embeddings
    let embeddings = vec![0.1, 0.2, 0.3];

    HttpResponse::Ok().json(embeddings)
}

// Lets save chatgpt style chat history at this endpoint
#[post("/chat")]
async fn save_chat(payload: web::Json<Chat>) -> HttpResponse {
    // Load the pre-trained model

    // Stub for embeddings
    HttpResponse::Ok().json(payload)
}

#[get("/chat/{id}")]
async fn get_chat(params: web::Path<(String,)>) -> impl Responder {
    let id = &params.0;
    println!("ID: {}", id);
    let chat = Chat {
        message: "Hello".to_string(),
    };
    HttpResponse::Ok().json(chat)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(embeddings))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
