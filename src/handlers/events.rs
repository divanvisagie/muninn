use std::time::Duration;

use actix_web::{web, HttpResponse};
use rumqttc::MqttOptions;
use serde::Serialize;
use tracing::{error, info};

#[derive(Serialize)]
pub struct MessageEvent {
    pub username: String,
    pub hash: String,
    pub chat_id: i64,
}

pub async fn test_mtqq(params: web::Path<(String,)>) -> HttpResponse {
    let username = &params.0.clone();

    let chat = rmp_serde::to_vec(&MessageEvent {
        username: username.clone(),
        hash: "hash".to_string(),
        chat_id: 1,
    })
    .unwrap();

    info!("Sending message to mqtt");

    let mut mqttoptions = MqttOptions::new("muninn", "127.0.0.1", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = rumqttc::AsyncClient::new(mqttoptions, 10);
    match client
        .publish(
            "messages/assistant",
            rumqttc::QoS::AtLeastOnce,
            false,
            chat.clone(),
        )
        .await
    {
        Ok(_) => {
            info!("Message sent");
        }
        Err(e) => {
            error!("Error sending message {}", e);
        }
    };

    while let Ok(notification) = eventloop.poll().await {
        match notification {
            rumqttc::Event::Incoming(incoming) => match incoming {
                rumqttc::Packet::PubAck(_) => {
                    info!("PubAck received");
                    break;
                }
                _ => {}
            },
            _ => {}
        }
    }
    HttpResponse::Ok().json(chat)
}
