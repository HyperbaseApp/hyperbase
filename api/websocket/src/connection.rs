use anyhow::{Error, Result};
use tokio::sync::mpsc;

use crate::{
    message::{Message, Target},
    session::UserSession,
    ConnectionId,
};

pub enum Connection {
    Connect {
        user_session: UserSession,
        target: Target,
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
        user_session: UserSession,
        target: Target,
        connection_id: ConnectionId,
        connection_tx: mpsc::UnboundedSender<Message>,
    ) -> Result<()> {
        self.tx
            .send(Connection::Connect {
                user_session,
                target,
                connection_id,
                connection_tx,
            })
            .map_err(|err| Error::from(err))?;
        Ok(())
    }

    pub fn disconnect(&self, connection_id: ConnectionId) -> Result<()> {
        self.tx
            .send(Connection::Disconnect(connection_id))
            .map_err(|err| Error::from(err))?;
        Ok(())
    }
}
