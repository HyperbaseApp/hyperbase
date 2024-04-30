use anyhow::{Error, Result};
use tokio::sync::mpsc;

use crate::message::Message;

#[derive(Clone)]
pub struct WebSocketBroadcaster {
    broadcast_tx: mpsc::UnboundedSender<Message>,
}

impl WebSocketBroadcaster {
    pub fn new(broadcast_tx: mpsc::UnboundedSender<Message>) -> Self {
        Self { broadcast_tx }
    }

    pub fn broadcast(&self, message: Message) -> Result<()> {
        self.broadcast_tx
            .send(message)
            .map_err(|err| Error::from(err))?;
        Ok(())
    }
}
