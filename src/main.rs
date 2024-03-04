use std::sync::Arc;

use actix_web::{web, App, HttpServer};
use clients::embeddings::OpenAiEmbeddingsClient;
use handlers::{chat::{get_chat, get_context, save_chat, search_chat}, summary::get_summary};
use repos::messages::FsMessageRepo;
use tokio::sync::Mutex;


mod clients;
mod handlers;
mod repos;
mod services;

struct Resources {
    pub message_repo: Arc<Mutex<dyn repos::messages::MessageRepo>>,
    pub embeddings_client: Arc<Mutex<dyn clients::embeddings::EmbeddingsClient>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    
    let open_ai_embeddings_client = Arc::new(Mutex::new(OpenAiEmbeddingsClient::new()));
    let message_repo = Arc::new(Mutex::new(FsMessageRepo::new()));

    let resources = Resources {
        message_repo,
        embeddings_client: open_ai_embeddings_client,
    };

    let data = web::Data::new(resources);

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .route("/api/v1/chat/{username}", web::post().to(save_chat))
            .route(
                "/api/v1/chat/{username}/context",
                web::get().to(get_context),
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
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}



