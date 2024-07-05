use std::sync::Arc;

use actix_web::{web, App, HttpServer};
use anyhow::Result;
use clients::embeddings::ollama::OllamaEmbeddingsClient;
use handlers::{
    chat::{get_chat, get_context_with, save_chat, search_chat},
    events::test_mtqq,
    summary::get_summary,
    user_attributes::{get_attribute, save_attribute},
};
use repos::{attributes::FsAttributeRepo, messages::FsMessageRepo};
use tokio::sync::Mutex;

mod clients;
mod handlers;
mod repos;
mod scheduler;
mod services;

struct Resources {
    message_repo: Arc<Mutex<dyn repos::messages::MessageRepo>>,
    embeddings_client: Arc<Mutex<dyn clients::embeddings::EmbeddingsClient>>,
    user_attributes_repo: Arc<Mutex<FsAttributeRepo>>,
}

impl Resources {
    fn new() -> Self {
        let client = OllamaEmbeddingsClient::new(&Some("all-minilm".to_string()));
        Resources {
            message_repo: Arc::new(Mutex::new(FsMessageRepo::new())),
            embeddings_client: Arc::new(Mutex::new(client)),
            user_attributes_repo: Arc::new(Mutex::new(FsAttributeRepo::new())),
        }
    }
}
async fn start_web_server(resources: Resources) -> Result<()> {
    let data = web::Data::new(resources);

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .route("/api/v1/chat/{username}", web::post().to(save_chat))
            .route(
                "/api/v1/chat/{username}/context",
                web::post().to(get_context_with),
            )
            .route("/api/v1/chat/{username}/{id}", web::get().to(get_chat))
            .route(
                "/api/v1/chat/{username}/search",
                web::post().to(search_chat),
            )
            .route(
                "/api/v1/summary/{username}/{date}",
                web::get().to(get_summary),
            )
            .route(
                "/api/v1/attribute/{username}",
                web::post().to(save_attribute),
            )
            .route(
                "/api/v1/attribute/{username}/{attribute}",
                web::get().to(get_attribute),
            )
            .route("/api/v1/events/{username}", web::get().to(test_mtqq))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await?;

    Ok(())
}

#[actix_web::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let model = Some("all-minilm".to_string());
    let open_ai_embeddings_client = Arc::new(Mutex::new(OllamaEmbeddingsClient::new(&model)));
    let message_repo = Arc::new(Mutex::new(FsMessageRepo::new()));

    let resources = Resources {
        message_repo,
        embeddings_client: open_ai_embeddings_client,
        user_attributes_repo: Arc::new(Mutex::new(FsAttributeRepo::new())),
    };

    start_web_server(resources).await
}
