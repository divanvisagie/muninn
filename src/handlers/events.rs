use std::time::Duration;

use actix_web::{web, HttpResponse};
use rumqttc::MqttOptions;
use serde::Serialize;
use tracing::{error, info};
use crate::repos::attributes::AttributeRepo;
use sha2::{Digest, Sha256};

use crate::repos::messages::ChatModel;
use crate::Resources;

#[derive(Clone, Debug, Serialize)]
pub struct MessageEvent {
    pub username: String,
    pub hash: String,
    pub chat_id: i64,
}

pub async fn test_mtqq(
    resources: web::Data<Resources>,
    params: web::Path<(String,)>,
) -> HttpResponse {
    let username = &params.0.clone();
    let attr = "telegram_chat_id".to_string();
    let chat_id = resources
        .user_attributes_repo
        .lock()
        .await
        .get_attribute(&username, &attr)
        .await;

    //create hash of message
    let content = "Event test message".to_string();
    let hash = Sha256::digest(content.as_bytes());
    let timestamp = chrono::Utc::now().timestamp();
    let chat = ChatModel {
        role: "assistant".to_string(),
        content,
        hash: format!("{:x}", hash),
        embedding: None,
        timestamp,
    };
    let date = chrono::Utc::now().date_naive();
    resources.message_repo.lock().await.save_chat(date, username.clone(), chat);

    let chat_id = match chat_id {
        Ok(chat_id) => chat_id.value,
        Err(_) => {
            error!("Error getting chat id");
            return HttpResponse::InternalServerError().finish();
        }
    };

    // convert chatid from string to i64
    let chat_id = chat_id.parse::<i64>().unwrap();

    let chat = rmp_serde::to_vec(&MessageEvent {
        username: username.clone(),
        hash: format!("{:x}", hash),
        chat_id,
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
