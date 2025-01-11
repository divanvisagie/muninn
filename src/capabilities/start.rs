use async_trait::async_trait;

use crate::handlers::handle_request_message::RequestMessage;

use super::Capability;

pub struct StartCapability {

}

impl StartCapability {
    pub fn new() -> Self {
        StartCapability {}
    }
}

#[async_trait]
impl Capability for StartCapability {
    fn get_name(&self) -> String {
        "StartCapability".to_string()
    }

    async fn check(&self, req: &RequestMessage) -> f32 {
        if req.text.as_str() == "/start" {
            return 1.0;
        }
        0.0
    }

    async fn execute(&self, _message: &RequestMessage) -> anyhow::Result<()> {
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::handle_request_message::RequestMessage;

    #[tokio::test]
    async fn test_check() {
        let capability = StartCapability::new();
        let req = RequestMessage::new(1, "/start".to_string());
        let result = capability.check(&req).await;
        assert_eq!(result, 1.0);
    }

}
