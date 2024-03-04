use actix_web::{web, HttpResponse};
use tracing::error;

use crate::{services::summary::SummaryService, Resources};

pub async fn get_summary(
    resources: web::Data<Resources>,
    params: web::Path<(String, String)>,
) -> HttpResponse {
    let resources = resources.into_inner();

    let summary_handler = SummaryService {
        message_repo: resources.message_repo.clone(),
        embedding_client: resources.embeddings_client.clone(),
    };

    let username = &params.0.clone();
    let date = &params.1.clone();
    let summary = summary_handler.summarize_chats_for_user_for_date(username.clone(), date.clone()).await;
    let summary = match summary {
        Ok(summary) => summary,
        Err(_) => {
            error!("Error getting summary");
            return HttpResponse::InternalServerError().finish();
        }
    };
    HttpResponse::Ok().json(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repos::messages::{get_path_for_date, ChatModel, FsMessageRepo};
    use crate::{clients::embeddings::MockEmbeddingsClient, repos::messages::MessageRepo};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_summarize_chats_for_user_for_date() {

        let mut repo = FsMessageRepo::new();
        let user = "test_user_summary".to_string();
        let date_string = "2021-01-01".to_string();
        let chat = ChatModel {
            role: "user".to_string(),
            content: "Hello".to_string(),
            hash: "123".to_string(),
            embedding: vec![0.0, 0.0, 0.0],
            timestamp: chrono::Utc::now().timestamp(),
        };

        let date = chrono::NaiveDate::parse_from_str(&date_string, "%Y-%m-%d").unwrap();

        let path = get_path_for_date(user.clone(), date);
        let _ = std::fs::remove_dir_all(path);

        repo.save_chat(date, user.clone(), chat);

        let handler = SummaryService {
            message_repo: Arc::new(Mutex::new(repo)),
            embedding_client: Arc::new(Mutex::new(MockEmbeddingsClient::new())),
        };

        let result = handler
            .summarize_chats_for_user_for_date(user.clone(), date_string.clone())
            .await;

        assert_eq!(result.unwrap().len(), 1);
    }
}
