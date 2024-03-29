use std::time::Duration;

use actix_web::{web, HttpResponse};
use rumqttc::MqttOptions;
use tracing::{error, info};

pub async fn test_mtqq(params: web::Path<(String,)>) -> HttpResponse {
    let username = &params.0.clone();
    let chat = format!("Hello {}", username);

    let mut mqttoptions = MqttOptions::new("rumqtt-async", "127.0.0.1", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (mut client, mut eventloop) = rumqttc::AsyncClient::new(mqttoptions, 10);
    match client
        .publish(
            "messages/assistant",
            rumqttc::QoS::AtLeastOnce,
            false,
            chat.as_bytes(),
        )
        .await
    {
        Ok(_) => {
            info!("Message sent");
        }
        Err(e) => {
            error!("Error sending message");
        }
    };
    while let Ok(notification) = eventloop.poll().await {
        println!("Received = {:?}", notification);
        match notification {
            rumqttc::Event::Incoming(incoming) => {
                println!("Incoming = {:?}", incoming);
                match incoming {
                    rumqttc::Packet::Publish(publish) => {
                        println!("Publish = {:?}", publish);
                    }
                    rumqttc::Packet::PubAck(_) => {
                        info!("PubAck");
                        break;
                    }
                    _ => {}
                }
            }
            rumqttc::Event::Outgoing(outgoing) => {
                println!("Outgoing = {:?}", outgoing);
            }
        }
    }
    HttpResponse::Ok().json(chat)
}
