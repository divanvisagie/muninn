use anyhow::Result;
use async_trait::async_trait;

use crate::handlers::handle_request_message::RequestMessage;

#[async_trait]
pub trait Layer: Send {
    async fn execute(&mut self, req: &RequestMessage) -> Result<()>;
}


pub mod security;
pub mod selector;
