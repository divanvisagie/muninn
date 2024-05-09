use chrono::NaiveDate;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::repos::messages::MessageRepo;

pub struct SummaryService {
    pub message_repo: Arc<Mutex<dyn MessageRepo>>,
    pub embedding_client: Arc<Mutex<dyn crate::clients::embeddings::EmbeddingsClient>>,
}

impl SummaryService {
    pub async fn summarize_chats_for_user_for_date(
        &self,
        user: String,
        date_str: String,
    ) -> Result<Vec<String>, ()> {
        let date = match NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
            Ok(date) => date,
            Err(_) => {
                return Err(());
            }
        };

        let messages = self
            .message_repo
            .lock()
            .await
            .get_all_for_user_on_day(user.clone(), date);

        // convert to message type

        match messages {
            Ok(messages) => {
                let mut summaries = vec![];
                for message in messages {
                    summaries.push(message.content.clone());
                }
                return Ok(summaries);
            }
            Err(_) => {
                return Err(());
            }
        }
    }
}
