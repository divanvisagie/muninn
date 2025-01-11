use anyhow::Result;
use axum::{routing::get, routing::post, Router};
use handlers::handle_request_message::handle_request_message;

mod clients;
mod handlers;
mod repos;
mod services;
mod layers;
mod capabilities;

#[derive(Clone)]
#[allow(dead_code)]
struct Resources {
    bot_name: String
}

pub async fn ping() -> String {
    "pong".to_string()
}

impl Resources {
    fn new() -> Self {
        Resources {
            bot_name: "Muninn".to_string()
        }
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
    
    let resources = Resources::new();

    start_web_server(resources).await
}
