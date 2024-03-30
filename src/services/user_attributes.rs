use std::{collections::HashMap, sync::Arc};

use serde::Deserialize;
use tokio::sync::Mutex;
use tracing::info;

use crate::repos::attributes::AttributeRepo;

#[derive(Deserialize)]
pub struct AttributeRequest {
    pub attribute: String,
    pub value: String,
}

pub struct UserAttributeService {
    pub attribute_repo: Arc<Mutex<dyn AttributeRepo>>,
}

impl UserAttributeService {
    pub async fn save_attribute(
        &mut self,
        username: &String,
        attribute: &String,
        value: &String,
    ) -> Result<(), ()> {
        self.attribute_repo
            .lock()
            .await
            .save_attribute(username, attribute, value)
            .await
            .map_err(|_| ())?;

        info!(
            "Saved attribute {} for user {} as {}",
            attribute, username, value
        );

        Ok(())
    }

    pub async fn get_attribute(&self, username: &String, attribute: &String) -> Result<String, ()> {
        let attribute = self
            .attribute_repo
            .lock()
            .await
            .get_attribute(username, attribute)
            .await
            .map_err(|_| ())?;

        Ok(attribute.value)
    }
}
