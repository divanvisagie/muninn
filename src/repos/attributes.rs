use std::collections::HashMap;

use async_trait::async_trait;
use tracing::error;

#[allow(dead_code)]
pub struct AttributeModel {
    pub attribute: String,
    pub value: String,
}

#[async_trait]
pub trait AttributeRepo {
    async fn save_attribute(
        &mut self,
        user: &String,
        attribute: &String,
        value: &String,
    ) -> Result<AttributeModel, ()>;
    async fn get_attribute(&mut self, user: &String, id: &String) -> Result<AttributeModel, ()>;
}

pub struct FsAttributeRepo {
    // Username: Attribute: Value
    memory: HashMap<String, HashMap<String, String>>,
}

#[allow(dead_code)]
impl FsAttributeRepo {
    pub fn new() -> Self {
        FsAttributeRepo {
            memory: HashMap::new(),
        }
    }
}

#[async_trait]
impl AttributeRepo for FsAttributeRepo {
    async fn save_attribute(
        &mut self,
        user: &String,
        attribute: &String,
        value: &String,
    ) -> Result<AttributeModel, ()> {
        if !self.memory.contains_key(user) {
            self.memory.insert(user.clone(), HashMap::new());
        }
        let user_attributes = self.memory.get_mut(user).unwrap();
        user_attributes.insert(attribute.clone(), value.clone());

        let user_attributes_save_file_path = get_root_path(&user).join("attributes.json");
        // Hashmap from attributes file
        let mut hm: HashMap<String, String> =
            match std::fs::read_to_string(&user_attributes_save_file_path) {
                Ok(content) => match serde_json::from_str(&content) {
                    Ok(hm) => hm,
                    Err(e) => {
                        error!("Error deserializing file: {}", e);
                        HashMap::new()
                    }
                },
                Err(_) => HashMap::new(),
            };
        // Insert the new attribute
        hm.insert(attribute.clone(), value.clone());
        // Serialize the hashmap
        let serialized = serde_json::to_string(&hm).unwrap();
        // Write the serialized hashmap to the file
        match std::fs::write(&user_attributes_save_file_path, serialized) {
            Ok(_) => (),
            Err(e) => {
                error!("Error writing to file: {}", e)
            }
        }

        Ok(AttributeModel {
            attribute: attribute.clone(),
            value: value.clone(),
        })
    }

    async fn get_attribute(&mut self, user: &String, id: &String) -> Result<AttributeModel, ()> {
        let user_attributes = self.memory.get(user);
        if user_attributes.is_none() {
            let value = get_from_file(user, id);
            // if that still fails we return an error
            if let None = value {
                return Err(());
            }

            return Ok(AttributeModel {
                attribute: id.clone(),
                value: value.unwrap().clone(),
            });
        }
        let user_attributes = user_attributes.unwrap();
        let value = user_attributes.get(id);

        if value.is_none() {
            let value = get_from_file(user, id);
            // if that still fails we return an error
            if let None = value {
                return Err(());
            }

            return Ok(AttributeModel {
                attribute: id.clone(),
                value: value.unwrap().clone(),
            });
        }

        Ok(AttributeModel {
            attribute: id.clone(),
            value: value.unwrap().clone(),
        })
    }
}
fn get_from_file(user: &String, id: &String) -> Option<String> {
    // if the value is not in memory we check the file system
    let user_attributes_save_file_path = get_root_path(user).join("attributes.json");
    let hm: HashMap<String, String> = match std::fs::read_to_string(&user_attributes_save_file_path)
    {
        Ok(content) => serde_json::from_str(&content).unwrap(),
        Err(_) => HashMap::new(),
    };
    let value = hm.get(id);
    match value {
        Some(value) => Some(value.clone()),
        None => None,
    }
}
fn get_root_path(user: &String) -> std::path::PathBuf {
    let dir = match std::env::var("MESSAGE_STORAGE_PATH") {
        Ok(val) => std::path::PathBuf::from(val),
        Err(_) => dirs::data_local_dir().unwrap(),
    };

    let path = dir.join("muninn").join(user.clone());
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_save_attribute() {
        let mut repo = FsAttributeRepo::new();
        let user = "test_user".to_string();
        let attribute = "test_attribute".to_string();
        let value = "test_attribute_value".to_string();

        let result = repo.save_attribute(&user, &attribute, &value).await;
        assert!(result.is_ok());
        let result = repo.get_attribute(&user, &attribute).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.attribute, attribute);
        assert_eq!(result.value, value);
    }

    #[tokio::test]
    async fn test_when_value_on_disk_only() {
        // write test attribute to disk
        let user = "test_disk_user".to_string();
        let attribute = "test_attribute".to_string();

        let path = get_root_path(&user).join("attributes.json");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap(); // create directory if it does not exist
        let mut hashmap = HashMap::new();
        hashmap.insert(attribute.clone(), "test_disk_value".to_string());
        let content = serde_json::to_string(&hashmap).unwrap();
        std::fs::write(path, content).unwrap();

        let mut repo = FsAttributeRepo::new();
        let result = repo.get_attribute(&user, &attribute).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.attribute, attribute);
        assert_eq!(result.value, "test_disk_value");
    }
}
