
use std::sync::Arc;

use actix_web::{web, App, HttpServer};
use clients::embeddings::BarnstokkrClient;
use handlers::{chat::{get_chat, get_context, save_chat, search_chat}, events::test_mtqq, summary::get_summary, user_attributes::{get_attribute, save_attribute}};
use repos::messages::FsMessageRepo;
use services::user_attributes::UserAttributeService;
use tokio::sync::Mutex;

mod clients;
mod handlers;
mod repos;
mod services;

struct Resources {
    message_repo: Arc<Mutex<dyn repos::messages::MessageRepo>>,
    embeddings_client: Arc<Mutex<dyn clients::embeddings::EmbeddingsClient>>,
    chat_client: Arc<Mutex<dyn clients::chat::ChatClient>>,
    user_attributes_service: Arc<Mutex<UserAttributeService>>,
}

impl Resources {
    fn new() -> Self {
        Resources {
            message_repo: Arc::new(Mutex::new(FsMessageRepo::new())),
            embeddings_client: Arc::new(Mutex::new(BarnstokkrClient::new())),
            chat_client: Arc::new(Mutex::new(clients::chat::GptClient::new())),
            user_attributes_service: Arc::new(Mutex::new(UserAttributeService::new())),
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    
    let open_ai_embeddings_client = Arc::new(Mutex::new(BarnstokkrClient::new()));
    let message_repo = Arc::new(Mutex::new(FsMessageRepo::new()));

    let resources = Resources {
        message_repo,
        embeddings_client: open_ai_embeddings_client,
        chat_client: Arc::new(Mutex::new(clients::chat::GptClient::new())),
        user_attributes_service: Arc::new(Mutex::new(UserAttributeService::new())),
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
            .route(
                "/api/v1/attribute/{username}",
                web::post().to(save_attribute),
            )
            .route(
                "/api/v1/attribute/{username}/{attribute}",
                web::get().to(get_attribute),
            )
            .route(
                "/api/v1/events/{username}",
                web::get().to(test_mtqq),
            )
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}



