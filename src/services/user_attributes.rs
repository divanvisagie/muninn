use std::collections::HashMap;

use serde::Deserialize;


#[derive(Deserialize)]
pub struct AttributeRequest {
    pub attribute: String,
    pub value: String,
}

pub struct UserAttributeService {
    memory: HashMap<String, HashMap<String, String>>,
}

impl UserAttributeService {
    pub fn new() -> Self {
        UserAttributeService {
            memory: HashMap::new(),
        }
    }

    pub async fn save_attribute(
        &mut self,
        username: &String,
        attribute: &String,
        value: &String,
    ) -> Result<(), ()> {
        let mut user_attributes = self.memory.get_mut(username);
        if user_attributes.is_none() {
            self.memory.insert(username.clone(), HashMap::new());
        }
        let user_attributes = self.memory.get_mut(username).unwrap();
        user_attributes.insert(attribute.clone(), value.clone());

        Ok(())
    }

    pub async fn get_attribute(
        &self,
        username: &String,
        attribute: &String,
    ) -> Result<String, ()> {
        let user_attributes = self.memory.get(username);
        if user_attributes.is_none() {
            return Err(());
        }
        let user_attributes = user_attributes.unwrap();
        let value = user_attributes.get(attribute);
        if value.is_none() {
            return Err(());
        }
        Ok(value.unwrap().clone())
    }
}
