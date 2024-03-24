use anyhow::{Error, Result};
use tokio::sync::mpsc;

use crate::server::{CollectionId, ConnectionId, Message, TokenId, UserId};

pub enum Connection {
    Connect {
        user_id: Option<UserId>,
        token_id: TokenId,
        collection_id: CollectionId,
        connection_id: ConnectionId,
        connection_tx: mpsc::UnboundedSender<Message>,
    },
    Disconnect(ConnectionId),
}

#[derive(Clone)]
pub struct WebSocketConnection {
    tx: mpsc::UnboundedSender<Connection>,
}

impl WebSocketConnection {
    pub fn new(tx: mpsc::UnboundedSender<Connection>) -> Self {
        Self { tx }
    }

    pub fn connect(
        &self,
        user_id: Option<UserId>,
        token_id: TokenId,
        collection_id: CollectionId,
        connection_id: ConnectionId,
        connection_tx: mpsc::UnboundedSender<Message>,
    ) -> Result<()> {
        self.tx
            .send(Connection::Connect {
                user_id,
                token_id,
                collection_id,
                connection_id,
                connection_tx,
            })
            .map_err(|err| Error::msg(err.to_string()))?;
        Ok(())
    }

    pub fn disconnect(&self, connection_id: ConnectionId) -> Result<()> {
        self.tx
            .send(Connection::Disconnect(connection_id))
            .map_err(|err| Error::msg(err.to_string()))?;
        Ok(())
    }
}
