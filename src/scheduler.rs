use std::sync::mpsc::{self, Receiver, Sender};
use std::{sync::Arc, time::Instant};
use tokio::sync::Mutex;
use tracing::info;

use crate::handlers::events::MessageEvent;

pub struct Scheduler {
    pub tasks: Arc<Mutex<Vec<(Instant, MessageEvent)>>>,
    sender: Sender<MessageEvent>,
    receiver: Arc<Mutex<Receiver<MessageEvent>>>,
    stop: Arc<Mutex<bool>>,
}

impl Scheduler {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        let tasks = Arc::new(Mutex::new(Vec::new()));
        Scheduler {
            tasks,
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
            stop: Arc::new(Mutex::new(false)),
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
        tokio::spawn(async move {
            loop {
                let mut tasks = tasks.lock().await;
                let stop = stop.lock().await;

                if *stop {
                    break;
                }

                if tasks.is_empty() {
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
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
        let mut scheduler = Scheduler::new();
        scheduler.start().await;

        let task = MessageEvent {
            username: "test".to_string(),
            hash: "test".to_string(),
            chat_id: 1,
        };
        scheduler.add_task(now + Duration::from_secs(5), task).await;

        // Advance time to just before the task should execute
        //time::pause();
        assert_eq!(scheduler.get_task_count().await, 1);
        //time::advance(Duration::from_secs(30)).await;
        //
        // sleep
        time::sleep(Duration::from_secs(15)).await;
        assert_eq!(scheduler.get_task_count().await, 0);
    }
}
