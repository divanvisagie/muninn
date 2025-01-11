use async_trait::async_trait;
use anyhow::Result;

use crate::handlers::handle_request_message::RequestMessage;

#[async_trait]
pub trait Capability: Send {
    fn get_name(&self) -> String {
        let raw = std::any::type_name::<Self>().to_string();
        let parts: Vec<&str> = raw.split("::").collect();
        parts.last().unwrap().to_string()
    }
    /// Returns a score between -1.0 and 1.0 indicating how well this capability
    /// can handle the request.
    async fn check(&self, req: &RequestMessage) -> f32;

    /// Executes the capability and returns a response message.
    async fn execute(&self, message: &RequestMessage) -> Result<()>;
}

pub mod test;
