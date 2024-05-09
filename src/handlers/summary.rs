use actix_web::{web, HttpResponse};
use tracing::error;

use crate::{services::summary::SummaryService, Resources};

pub async fn get_summary(
    resources: web::Data<Resources>,
    params: web::Path<(String, String)>,
) -> HttpResponse {
    let resources = resources.into_inner();

    let summary_service = SummaryService {
        message_repo: resources.message_repo.clone(),
        embedding_client: resources.embeddings_client.clone(),
    };

    let username = &params.0.clone();
    let date = &params.1.clone();
    let summary = summary_service.summarize_chats_for_user_for_date(username.clone(), date.clone()).await;
    let summary = match summary {
        Ok(summary) => summary,
        Err(_) => {
            error!("Error getting summary");
            return HttpResponse::InternalServerError().finish();
        }
    };
    HttpResponse::Ok().json(summary)
}

