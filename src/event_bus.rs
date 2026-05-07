use async_trait::async_trait;
use tokio::sync::broadcast;

use crate::{
    error::{AuthError, AuthResult},
    models::Event,
    traits::EventBus,
};

#[derive(Debug, Clone)]
pub struct InMemoryEventBus {
    tx: broadcast::Sender<Event>,
}

impl InMemoryEventBus {
    pub fn new(buffer: usize) -> Self {
        let (tx, _) = broadcast::channel(buffer);
        Self { tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.tx.subscribe()
    }
}

#[async_trait]
impl EventBus for InMemoryEventBus {
    async fn publish(&self, event: Event) -> AuthResult<()> {
        self.tx
            .send(event)
            .map_err(|err| AuthError::Storage(format!("event bus publish failed: {err}")))?;
        Ok(())
    }
}
