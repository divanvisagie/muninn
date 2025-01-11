use crate::{
    capabilities::{start::StartCapability, test::TestCapability, Capability},
    handlers::handle_request_message::RequestMessage,
};

use super::Layer;
use anyhow::Result;
use async_trait::async_trait;

pub enum CapabilityVariant {
    Test(TestCapability),
    Start(StartCapability),
}

impl CapabilityVariant {
    pub async fn check(&self, req: &RequestMessage) -> f32 {
        match self {
            CapabilityVariant::Test(capability) => capability.check(req).await,
            CapabilityVariant::Start(capability) => capability.check(req).await,
        }
    }

    pub async fn execute(&self, req: &RequestMessage) -> Result<()> {
        match self {
            CapabilityVariant::Test(capability) => capability.execute(req).await,
            CapabilityVariant::Start(capability) => capability.execute(req).await,
        }
    }

    pub fn get_name(&self) -> String {
        match self {
            CapabilityVariant::Test(capability) => capability.get_name(),
            CapabilityVariant::Start(capability) => capability.get_name(),
        }
    }
}

pub struct SelectorLayer {
    capabilities: Vec<CapabilityVariant>,
}

impl SelectorLayer {
    pub fn new() -> Self {
        let test = TestCapability::new();
        let start = StartCapability::new();
        SelectorLayer {
            capabilities: vec![
                CapabilityVariant::Test(test),
                CapabilityVariant::Start(start),
            ],
        }
    }
}

#[async_trait]
impl Layer for SelectorLayer {
    async fn execute(&mut self, req: &RequestMessage) -> Result<()> {
        let mut highest = 0.0;
        let mut highest_capability: Option<&CapabilityVariant> = None;

        for capability in &mut self.capabilities {
            let result = capability.check(req).await;
            if result > highest {
                highest = result;
                highest_capability = Some(capability);
            }
        }

        match highest_capability {
            Some(capability) => {
                println!(
                    "SelectorLayer: Executing capability: {}",
                    capability.get_name()
                );
                capability.execute(req).await?;
            }
            None => {
                println!("SelectorLayer: No capability found");
            }
        }
        println!("SelectorLayer: Executing security checks");
        Ok(())
    }
}
