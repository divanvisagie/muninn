use std::path::PathBuf;

use async_trait::async_trait;
use chrono::NaiveDate;
use tracing::error;

use crate::clients::{
    self,
    embeddings::{ollama::OllamaEmbeddingsClient, EmbeddingsClient},
};

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct ChatModel {
    pub role: String,
    pub content: String,
    pub hash: String,
    pub embedding: Option<Vec<f32>>,
    pub timestamp: i64,
}
pub struct FsMessageRepo {
    memory: std::collections::HashMap<(String, String), ChatModel>, // Update HashMap key to include user
}

#[async_trait]
pub trait MessageRepo: Send + Sync {
    fn save_chat(&mut self, date: NaiveDate, user: String, chat: ChatModel) -> ChatModel;
    fn get_chat(&mut self, user: String, id: String) -> Result<ChatModel, ()>; // Add user parameter
    async fn embeddings_search_for_user(
        &self,
        user: String,
        query_vector: Vec<f32>,
    ) -> Vec<(f32, ChatModel)>;
    fn get_all_for_user(&self, user: String) -> Result<Vec<ChatModel>, ()>;
    fn get_all_for_user_on_day(&self, user: String, date: NaiveDate) -> Result<Vec<ChatModel>, ()>;
}

impl FsMessageRepo {
    pub fn new() -> FsMessageRepo {
        FsMessageRepo {
            memory: std::collections::HashMap::new(),
        }
    }
}

fn cosine_similarity(v1: &Vec<f32>, v2: &Vec<f32>) -> f32 {
    let dot_product = v1.iter().zip(v2).map(|(a, b)| a * b).sum::<f32>();
    let magnitude_v1 = (v1.iter().map(|a| a.powi(2)).sum::<f32>()).sqrt();
    let magnitude_v2 = (v2.iter().map(|a| a.powi(2)).sum::<f32>()).sqrt();
    let magnitude_product = magnitude_v1 * magnitude_v2;
    dot_product / magnitude_product
}

fn get_root_path(user: String) -> std::path::PathBuf {
    let dir = match std::env::var("MESSAGE_STORAGE_PATH") {
        Ok(val) => std::path::PathBuf::from(val),
        Err(_) => dirs::data_local_dir().unwrap(),
    };

    let path = dir.join("muninn").join(user.clone());
    path
}
pub fn get_path_for_date(user: String, date: NaiveDate) -> std::path::PathBuf {
    let path = get_root_path(user.clone()).join(format!("{}", date.format("%Y-%m-%d")));
    path
}

fn get_from_fs(path: PathBuf) -> Vec<ChatModel> {
    let chats: Vec<ChatModel> = match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap(),
        Err(_) => vec![],
    };
    chats
}

#[async_trait]
impl MessageRepo for FsMessageRepo {
    fn save_chat(&mut self, date: NaiveDate, user: String, chat: ChatModel) -> ChatModel {
        let key = (chat.hash.clone(), user.clone());
        self.memory.insert(key, chat.clone());

        let path = get_path_for_date(user.clone(), date).join("messages.json");

        let mut chats = get_from_fs(path.clone());
        chats.push(chat.clone());

        std::fs::create_dir_all(path.parent().unwrap()).unwrap(); // create directory if it does not exist
        let serialized = serde_json::to_string(&chats).unwrap();

        match std::fs::write(&path, serialized) {
            Ok(_) => (),
            Err(e) => {
                error!("Error writing to file: {}", e)
            }
        }
        chat
    }

    fn get_chat(&mut self, user: String, id: String) -> Result<ChatModel, ()> {
        let key = (id, user.clone());
        let path = get_path_for_date(user.clone(), chrono::Local::now().date_naive())
            .join("messages.json");

        match self.memory.get(&key) {
            Some(chat) => Ok(chat.clone()),
            None => {
                let chats = get_from_fs(path);
                for chat in chats {
                    let key = (chat.hash.clone(), user.clone());
                    self.memory.insert(key, chat.clone());
                }
                match self.memory.get(&key) {
                    Some(chat) => Ok(chat.clone()),
                    None => {
                        error!("Chat not found");
                        Err(())
                    }
                }
            }
        }
    }
    fn get_all_for_user(&self, user: String) -> Result<Vec<ChatModel>, ()> {
        let path = get_root_path(user.clone());
        // Find all the subdirectories
        let date_folders = match std::fs::read_dir(&path) {
            Ok(val) => val,
            Err(_) => return Ok(vec![]),
        };

        let date_folders = date_folders
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .collect::<Vec<PathBuf>>();

        let mut date_folders = date_folders
            .iter()
            .filter_map(|x| {
                x.file_name()
                    .and_then(|name| name.to_str())
                    .and_then(|date_str| NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok())
            })
            .collect::<Vec<NaiveDate>>();

        // Sort the dates in ascending order
        date_folders.sort();

        // Get from fs for each date and add to the list of chat models
        let mut chats: Vec<ChatModel> = vec![];
        for date in date_folders.iter() {
            let path = get_path_for_date(user.clone(), *date).join("messages.json");
            // Assuming get_from_fs returns a Vec<ChatModel>
            let chat_models = get_from_fs(path);
            for chat in chat_models {
                chats.push(chat);
            }
        }
        Ok(chats)
    }

    async fn embeddings_search_for_user(
        &self,
        user: String,
        query_vector: Vec<f32>,
    ) -> Vec<(f32, ChatModel)> {
        let chats = self.get_all_for_user(user.clone());
        let chats = match chats {
            Ok(val) => val,
            Err(_) => return vec![],
        };

        let mut ranked_chats: Vec<(f32, ChatModel)> = vec![];
        let model = Some("all-minilm".to_string());
        let embedding_client = clients::embeddings::ollama::OllamaEmbeddingsClient::new(&model);

        for chat in chats {
            let chat_embedding = match &chat.embedding {
                Some(val) => val.clone(),
                None => {
                    let embedding = embedding_client
                        .get_embeddings(&[chat.content.as_str()])
                        .await;
                    match embedding {
                        Ok(val) => val[0].clone(),
                        Err(_) => {
                            error!("Failed to get embeddings");
                            return vec![];
                        }
                    }
                }
            };
            let similarity = cosine_similarity(&chat_embedding, &query_vector);
            ranked_chats.push((similarity, chat));
        }

        ranked_chats
    }

    fn get_all_for_user_on_day(&self, user: String, date: NaiveDate) -> Result<Vec<ChatModel>, ()> {
        let path = get_path_for_date(user.clone(), date).join("messages.json");
        let chats = get_from_fs(path);
        Ok(chats)
    }
}
