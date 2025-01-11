use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use axum::{routing::get, routing::post, Router};
use clients::{
    chat::{ChatClient, GptClient},
    embeddings::OllamaEmbeddingsClient,
};
use handlers::handle_request_message::handle_request_message;
use repos::{attributes::FsAttributeRepo, messages::FsMessageRepo};
use tokio::{net::TcpListener, sync::Mutex};

mod clients;
mod handlers;
mod repos;
mod services;

#[derive(Clone)]
struct Resources {}

pub async fn ping() -> String {
    "pong".to_string()
}

impl Resources {
    fn new() -> Self {
        Resources {}
    }
}
async fn start_web_server(resources: Resources) -> Result<()> {
    let app = Router::new()
        .route("/api/v1/message", post(handle_request_message))
        .route("/ping", get(ping))
        .with_state(resources);

    let listener = tokio::net::TcpListener::bind("localhost:8080").await.unwrap();
    axum::serve(listener, app).await?;
    Ok(())
}

#[tokio::main]
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
