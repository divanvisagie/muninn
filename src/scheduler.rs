use std::sync::mpsc::{self, Receiver, Sender};
use std::{sync::Arc, time::Instant};
use tokio::sync::Mutex;
use tracing::info;

use crate::handlers::events::MessageEvent;

pub struct Scheduler {
    tasks: Arc<Mutex<Vec<(Instant, MessageEvent)>>>,
    sender: Sender<MessageEvent>,
    receiver: Arc<Mutex<Receiver<MessageEvent>>>,
    stop: Arc<Mutex<bool>>,
    sleep_duration: Arc<Mutex<u64>>
}

impl Scheduler {
    pub fn new(sleep_duration: u64) -> Self {
        let (sender, receiver) = mpsc::channel();
        let tasks = Arc::new(Mutex::new(Vec::new()));
        Scheduler {
            tasks,
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
            stop: Arc::new(Mutex::new(false)),
            sleep_duration: Arc::new(Mutex::new(sleep_duration))
        }
    }

    pub async fn add_task(&mut self, when: Instant, task: MessageEvent) {
        self.tasks.lock().await.push((when, task));
    }

    pub async fn stop(&self) {
        let mut stop = self.stop.lock().await;
        *stop = true;
    }

    pub async fn get_task_count(&self) -> usize {
        self.tasks.lock().await.len()
    }

    pub async fn start(&mut self) {
        let stop = self.stop.clone();
        let tasks = self.tasks.clone();
        let sleep_duration = self.sleep_duration.clone();

        tokio::spawn(async move {
            loop {
                let mut tasks = tasks.lock().await;
                let stop = stop.lock().await;
                let sleep_duration = sleep_duration.lock().await;

                if *stop {
                    break;
                }

                if tasks.is_empty() {
                    tokio::time::sleep(tokio::time::Duration::from_secs(*sleep_duration)).await;
                    continue;
                }

                if let Some((when, task)) = tasks.first() {
                    let now = Instant::now();
                    if now >= *when {
                        info!("Executing task: {:?}", task);
                        // self.sender.send(task.clone()).unwrap();
                        //    tasks.lock().await.remove(0);
                        tasks.remove(0);
                    }
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::events::MessageEvent;
    use tokio::time::{self, Duration};

    #[tokio::test]
    async fn test_scheduler_with_single_item() {
        let now = Instant::now();
        let mut scheduler = Scheduler::new(1);
        scheduler.start().await;

        let task = MessageEvent {
            username: "test".to_string(),
            hash: "test".to_string(),
            chat_id: 1,
        };
        scheduler.add_task(now + Duration::from_secs(5), task).await;

        assert_eq!(scheduler.get_task_count().await, 1);
        time::sleep(Duration::from_secs(10)).await;
        assert_eq!(scheduler.get_task_count().await, 0);
    }
}
