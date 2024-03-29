use actix_web::{web, HttpResponse};
use tracing::error;

use crate::{
    services::user_attributes::{AttributeRequest, UserAttributeService},
    Resources,
};
pub async fn save_attribute(
    resources: web::Data<Resources>,
    params: web::Path<(String,)>,
    payload: web::Json<AttributeRequest>,
) -> HttpResponse {
    let resources = resources.into_inner();
    let username = &params.0.clone();
    let attribute = &payload.attribute.clone();
    let value = &payload.value.clone();
    let attribute = resources
        .user_attributes_service
        .lock()
        .await
        .save_attribute(username, attribute, value)
        .await;

    let attribute = match attribute {
        Ok(attribute) => attribute,
        Err(_) => {
            error!("Error saving attribute");
            return HttpResponse::InternalServerError().finish();
        }
    };
    HttpResponse::Ok().json(attribute)
}

pub async fn get_attribute(
    resources: web::Data<Resources>,
    params: web::Path<(String, String)>,
) -> HttpResponse {
    let resources = resources.into_inner();
    let username = &params.0.clone();
    let attribute = &params.1.clone();
    let attribute = resources
        .user_attributes_service
        .clone()
        .lock()
        .await
        .get_attribute(username, attribute)
        .await;

    let attribute = match attribute {
        Ok(attribute) => attribute,
        Err(_) => {
            error!("Error getting attribute");
            return HttpResponse::InternalServerError().finish();
        }
    };
    HttpResponse::Ok().json(attribute)
}

#[cfg(test)]
mod tests {
    use actix_web::{http::StatusCode, test, web, App};
    use serde_json::json;

    use crate::{handlers::user_attributes::save_attribute, Resources};

    use super::get_attribute;

    #[actix::test]
    async fn test_save_attribute() {
        let resources = Resources::new();
        let mut app = test::init_service(App::new().app_data(web::Data::new(resources)).route(
            "/api/v1/user/{username}/attribute",
            web::post().to(save_attribute),
        ).route(
            "/api/v1/user/{username}/{attribute}",
            web::get().to(get_attribute),
        ))
        .await;

        let req = test::TestRequest::post()
            .uri("/api/v1/user/username/attribute")
            .set_json(&json!({"attribute": "test_attr", "value": "test"}))
            .to_request();

        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        // check if attribute is present in memory
        let resp = test::TestRequest::get()
            .uri("/api/v1/user/username/test_attr")
            .to_request();
        let resp = test::call_service(&mut app, resp).await;
        assert_eq!(resp.status(), StatusCode::OK);

    }

    #[actix::test]
    async fn test_get_attribute_when_absent() {
        let resources = Resources::new();
        let mut app = test::init_service(App::new().app_data(web::Data::new(resources)).route(
            "/api/v1/attributes/{username}/{attribute}",
            web::get().to(get_attribute),
        ))
        .await;

        let req = test::TestRequest::get()
            .uri("/api/v1/user/username/test_key")
            .to_request();

        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[actix::test]
    async fn test_get_attribute_when_present() {
        let resources = Resources::new();

        // Add attribute to memory
        resources
            .user_attributes_service
            .lock()
            .await
            .save_attribute(
                &"username".to_string(),
                &"test_key".to_string(),
                &"test_value".to_string(),
            )
            .await
            .unwrap();

        // Test for presence
        let mut app = test::init_service(App::new().app_data(web::Data::new(resources)).route(
            "/api/v1/attributes/{username}/{attribute}",
            web::get().to(get_attribute),
        ))
        .await;

        let req = test::TestRequest::get()
            .uri("/api/v1/attributes/username/test_key")
            .to_request();

        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = test::read_body(resp).await;
        assert_eq!(body, r#""test_value""#);
    }
}
