use std::{sync::Arc, time::Instant};
use tokio::sync::Mutex;
use std::sync::mpsc::{self, Sender, Receiver};
use tracing::info;

use crate::handlers::events::MessageEvent;

pub struct Scheduler {
    pub tasks: Vec<(Instant, MessageEvent)>,
    sender: Sender<MessageEvent>,
    receiver: Arc<Mutex<Receiver<MessageEvent>>>,
    stop: Arc<Mutex<bool>>,
}

impl Scheduler {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Scheduler {
            tasks: Vec::new(),
            sender, 
            receiver: Arc::new(Mutex::new(receiver)),
            stop: Arc::new(Mutex::new(false)),
        }
    }

    pub fn add_task(&mut self, when: Instant, task: MessageEvent) {
        self.tasks.push((when, task));
    }

    pub async fn stop(&self) {
        let mut stop = self.stop.lock().await;
        *stop = true;
    }

    pub async fn run(&mut self) {
        loop {
            {
                let stop = self.stop.lock().await;
                if *stop {
                    break;
                }
            }

            if self.tasks.is_empty() {
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                continue;
            }

            self.tasks.sort_by_key(|task| task.0); // Ensure the soonest task is first

            let now = Instant::now();
            let (scheduled_time, task) = self.tasks[0].clone();

            if now >= scheduled_time {
                info!("Executing: {:?}", task);
                self.tasks.remove(0);
            } else {
                let sleep_duration = scheduled_time.duration_since(now);
                tokio::time::sleep(sleep_duration).await;
            }
        }
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

        let task = MessageEvent {
            username: "test".to_string(),
            hash: "test".to_string(),
            chat_id: 1,
        };

        scheduler.add_task(now + Duration::from_secs(5), task);

        let mut scheduler = Arc::new(Mutex::new(scheduler));
        let mut scheduler_clone = Arc::clone(&scheduler);

        let handle = tokio::spawn(async move {
            scheduler_clone.lock().await.run().await;
        });

        // Advance time to just before the task should execute
        time::pause();
        time::advance(Duration::from_secs(4)).await;
        assert_eq!(scheduler.lock().await.tasks.len(), 1);
        // assert_eq!(scheduler.tasks.len(), 1);
        //
        // // Advance time to when the task should execute
        // time::advance(Duration::from_secs(2)).await;
        // assert_eq!(scheduler.tasks.len(), 0);
        //
        // // Stop the scheduler
        scheduler_clone.lock().await.stop().await;
        handle.await.unwrap();
    }
}
