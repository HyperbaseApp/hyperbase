use anyhow::{Error, Result};
use tokio::sync::mpsc;

use crate::server::{CollectionId, Message};

#[derive(Clone)]
pub struct WebSocketBroadcaster {
    broadcast_tx: mpsc::UnboundedSender<(CollectionId, Message)>,
}

impl WebSocketBroadcaster {
    pub fn new(broadcast_tx: mpsc::UnboundedSender<(CollectionId, Message)>) -> Self {
        Self { broadcast_tx }
    }

    pub fn broadcast(&self, collection_id: CollectionId, message: Message) -> Result<()> {
        self.broadcast_tx
            .send((collection_id, message))
            .map_err(|err| Error::msg(err.to_string()))?;
        Ok(())
    }
}
