use std::path::PathBuf;

use chrono::NaiveDate;
use tracing::error;

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct ChatModel {
    pub role: String,
    pub content: String,
    pub hash: String,
    pub embedding: Vec<f32>,
    pub timestamp: i64,
}
pub struct FsMessageRepo {
    memory: std::collections::HashMap<(String, String), ChatModel>, // Update HashMap key to include user
}

pub trait MessageRepo: Send + Sync {
    fn save_chat(&mut self, date: NaiveDate, user: String, chat: ChatModel) -> ChatModel;
    fn get_chat(&mut self, user: String, id: String) -> Result<ChatModel, ()>; // Add user parameter
    fn embeddings_search_for_user(
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

impl MessageRepo for FsMessageRepo {
    fn save_chat(&mut self, date: NaiveDate, user: String, chat: ChatModel) -> ChatModel {
        let key = (chat.hash.clone(), user.clone());
        self.memory.insert(key, chat.clone());

        let path = get_path_for_date(user.clone(), date).join("messages.json");

        let mut chats = get_from_fs(path.clone());
        chats.push(chat.clone());

        // append chat to file if it exists or create a new file
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
        let key = (id, user.clone()); // Create key using id and user
        let path = get_path_for_date(user.clone(), chrono::Local::now().date_naive())
            .join("messages.json");
        match self.memory.get(&key) {
            Some(chat) => Ok(chat.clone()),
            None => {
                let chats = get_from_fs(path);
                // put these in memory
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

    fn embeddings_search_for_user(
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
        for chat in chats {
            let similarity = cosine_similarity(&chat.embedding, &query_vector);
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

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;
    #[test]
    fn test_save_chat_and_get_chat() {
        let id = Uuid::new_v4().to_string();
        let chat = ChatModel {
            role: "user".to_string(),
            content: "Hello".to_string(),
            hash: id.clone(),
            embedding: vec![0.1, 0.2, 0.3],
            timestamp: chrono::Utc::now().timestamp(),
        };
        let expected_hash = id.clone();
        let expected_role = chat.role.clone();
        let expected_content = chat.content.clone();

        let mut repo = FsMessageRepo::new();
        let todays_date = chrono::Local::now().date_naive();
        repo.save_chat(todays_date, "test_user".to_string(), chat.clone()); // Pass user parameter

        let got_chat = repo.get_chat("test_user".to_string(), id).unwrap(); // Pass user parameter
        assert_eq!(got_chat.role, expected_role);
        assert_eq!(got_chat.content, expected_content);
        assert_eq!(got_chat.hash, expected_hash);
    }

    #[test]
    fn test_get_chat_when_no_user() {
        let id = Uuid::new_v4().to_string();
        let chat = ChatModel {
            role: "user".to_string(),
            content: "Hello".to_string(),
            hash: id.clone(),
            embedding: vec![0.1, 0.2, 0.3],
            timestamp: chrono::Utc::now().timestamp(),
        };
        let mut repo = FsMessageRepo::new();
        let today = chrono::Local::now().date_naive();
        repo.save_chat(today, "test_user".to_string(), chat.clone());

        let got_chat = repo.get_chat("test_user2".to_string(), id);

        //test that the result was an error
        assert!(got_chat.is_err());
    }

    #[test]
    fn test_get_when_there_is_no_chat() {
        let id = Uuid::new_v4().to_string();
        let chat = ChatModel {
            role: "user".to_string(),
            content: "Hello".to_string(),
            hash: id.clone(),
            embedding: vec![0.1, 0.2, 0.3],
            timestamp: chrono::Utc::now().timestamp(),
        };
        let mut repo = FsMessageRepo::new();
        let today = chrono::Local::now().date_naive();
        repo.save_chat(today, "test_user".to_string(), chat.clone());

        let got_chat = repo.get_chat("test_user".to_string(), uuid::Uuid::new_v4().to_string());

        //test that the result was an error
        assert!(got_chat.is_err());
    }

    #[test]
    fn test_embeddings_search_for_user() {
        let username = "test_user_embeddings".to_string();

        let path = get_root_path(username.to_string());
        let _ = std::fs::remove_dir_all(path);

        let mut repo = FsMessageRepo::new();
        let id = Uuid::new_v4().to_string();
        let chat = ChatModel {
            role: username.to_string(),
            content: "Hello".to_string(),
            hash: id.clone(),
            embedding: vec![0.1, 0.2, 0.3],
            timestamp: chrono::Utc::now().timestamp(),
        };
        let chat2 = ChatModel {
            role: username.to_string(),
            content: "Go away, you are not welcome".to_string(),
            hash: Uuid::new_v4().to_string(),
            embedding: vec![0.1, 0.2, 0.3],
            timestamp: chrono::Utc::now().timestamp(),
        };
        let date = chrono::Local::now().date_naive() - chrono::Duration::days(5);
        repo.save_chat(date, username.to_string(), chat.clone());
        let date = chrono::Local::now().date_naive() - chrono::Duration::days(2);
        repo.save_chat(date, username.to_string(), chat2.clone());

        let query_vector = vec![0.1, 0.2, 0.3];
        let results = repo.embeddings_search_for_user(username.to_string(), query_vector);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].1.content, "Hello");
        assert_eq!(results[1].1.content, "Go away, you are not welcome");
    }

    #[test]
    fn test_get_all_for_user() {
        let user = "test_user2".to_string();
        // delete the folder for user
        let path = get_root_path(user.clone());
        let _ = std::fs::remove_dir_all(path);

        // lets add some old date subdirectories
        let date = chrono::Local::now().date_naive() - chrono::Duration::days(2);
        let path = get_path_for_date(user.clone(), date);
        let _ = std::fs::create_dir_all(path);
        let date = chrono::Local::now().date_naive() - chrono::Duration::days(5);
        let path = get_path_for_date(user.clone(), date);
        let _ = std::fs::create_dir_all(path);

        let mut repo = FsMessageRepo::new();

        // add messages for this date in particular
        let chat = ChatModel {
            role: "user".to_string(),
            content: "Hello".to_string(),
            hash: Uuid::new_v4().to_string(),
            embedding: vec![0.1, 0.2, 0.3],
            timestamp: chrono::Utc::now().timestamp(),
        };
        let date = chrono::Local::now().date_naive() - chrono::Duration::days(5);
        repo.save_chat(date, user.clone(), chat.clone());
        let chat = ChatModel {
            role: "user".to_string(),
            content: "this is the second message".to_string(),
            hash: Uuid::new_v4().to_string(),
            embedding: vec![0.1, 0.2, 0.3],
            timestamp: chrono::Utc::now().timestamp(),
        };
        repo.save_chat(date, user.clone(), chat.clone());

        // set up a special user folder to only have subfolders
        // with dates in the past
        let _ = get_root_path(user.clone());

        let chat = ChatModel {
            role: "user".to_string(),
            content: "this is the second message".to_string(),
            hash: Uuid::new_v4().to_string(),
            embedding: vec![0.1, 0.2, 0.3],
            timestamp: chrono::Utc::now().timestamp(),
        };
        let date = chrono::Local::now().date_naive() - chrono::Duration::days(2);
        repo.save_chat(date, user.clone(), chat.clone());

        let chatsk = repo.get_all_for_user(user.clone());
        assert_eq!(chatsk.as_ref().unwrap().len(), 3);
        assert_eq!(chatsk.as_ref().unwrap()[0].content, "Hello");

        // assert_eq!(chatsk.unwrap()[1].content, "this is the second message");
    }
}
