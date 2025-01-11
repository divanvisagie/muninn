use crate::handlers::handle_request_message::RequestMessage;

use anyhow::Result;
use async_trait::async_trait;
use super::Layer;

pub struct SecurityLayer {
    next_layer: Box<dyn Layer>
}

impl SecurityLayer {
    pub fn new(next_layer: Box<dyn Layer>) -> Self {
        SecurityLayer {
            next_layer,
        }
    }
}

#[async_trait]
impl Layer for SecurityLayer {
    async fn execute(&mut self, req: &RequestMessage) -> Result<()> {
        println!("SecurityLayer: Executing security checks");
        self.next_layer.execute(req).await?;
        Ok(())
    }
}
