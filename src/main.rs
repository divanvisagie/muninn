use std::sync::Arc;

use actix_web::{web, App, HttpResponse, HttpServer};
use anyhow::Result;
use clients::{
    chat::{ChatClient, GptClient},
    embeddings::OllamaEmbeddingsClient,
};
use handlers::{
    handle_request_message::handle_request_message
};
use repos::{attributes::FsAttributeRepo, messages::FsMessageRepo};
use tokio::sync::Mutex;

mod clients;
mod handlers;
mod repos;
mod services;

struct Resources {
}

pub async fn ping() -> HttpResponse {
    HttpResponse::Ok().json("pong")
}

impl Resources {
    fn new() -> Self {
        Resources {
            // message_repo: Arc::new(Mutex::new(FsMessageRepo::new())),
            // embeddings_client: Arc::new(Mutex::new(OllamaEmbeddingsClient::new())),
            // user_attributes_repo: Arc::new(Mutex::new(FsAttributeRepo::new())),
        }
    }
}
async fn start_web_server(resources: Resources) -> Result<()> {
    let data = web::Data::new(resources);

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .route("/api/v1/message", web::post().to(handle_request_message))
            .route("/ping", web::get().to(ping))
    })
    .bind("localhost:8080")?
    .run()
    .await?;

    Ok(())
}

#[actix_web::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // let open_ai_embeddings_client = Arc::new(Mutex::new(OllamaEmbeddingsClient::new()));
    // let message_repo = Arc::new(Mutex::new(FsMessageRepo::new()));

    let resources = Resources {
        // message_repo,
        // embeddings_client: open_ai_embeddings_client,
        // user_attributes_repo: Arc::new(Mutex::new(FsAttributeRepo::new())),
    };

    start_web_server(resources).await
}

