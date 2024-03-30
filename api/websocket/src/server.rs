use std::time::Duration;

use ahash::{HashMap, HashMapExt, HashSet};
use anyhow::Result;
use hb_dao::{collection_rule::CollectionPermission, token::TokenDao};
use serde::Serialize;
use tokio::{select, sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::{
    broadcaster::WebSocketBroadcaster,
    connection::{Connection, WebSocketConnection},
    context::ApiWebSocketCtx,
    handler::WebSocketHandler,
};

pub type ConnectionId = Uuid;
pub type UserId = Uuid;
pub type TokenId = Uuid;
pub type CollectionId = Uuid;

#[derive(Serialize, Clone)]
pub struct Message {
    #[serde(skip_serializing)]
    pub collection_id: CollectionId,
    #[serde(skip_serializing)]
    pub data_id: Uuid,
    #[serde(skip_serializing)]
    pub created_by: UserId,

    kind: MessageKind,
    data: serde_json::Value,
}

impl Message {
    pub fn new(
        collection_id: CollectionId,
        data_id: Uuid,
        created_by: UserId,
        kind: MessageKind,
        data: serde_json::Value,
    ) -> Self {
        Self {
            collection_id,
            data_id,
            created_by,
            kind,
            data,
        }
    }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum MessageKind {
    InsertOne,
    UpdateOne,
    DeleteOne,
}

pub struct ApiWebSocketServer {
    ctx: ApiWebSocketCtx,

    sessions: HashMap<ConnectionId, mpsc::UnboundedSender<Message>>,
    user_sessions: HashMap<ConnectionId, (Option<UserId>, TokenId)>,
    subscribers: HashMap<CollectionId, HashSet<ConnectionId>>,

    connection_rx: mpsc::UnboundedReceiver<Connection>,
    broadcast_rx: mpsc::UnboundedReceiver<(CollectionId, Message)>,
}

impl ApiWebSocketServer {
    pub fn new(
        ctx: ApiWebSocketCtx,

        heartbeat_interval: &Duration,
        client_timeout: &Duration,
    ) -> (Self, WebSocketHandler, WebSocketBroadcaster) {
        hb_log::info(Some("⚡"), "ApiWebSocketServer: Initializing component");

        let (connection_tx, connection_rx) = mpsc::unbounded_channel();
        let (broadcast_tx, broadcast_rx) = mpsc::unbounded_channel();

        let connection = WebSocketConnection::new(connection_tx);
        let publisher = WebSocketBroadcaster::new(broadcast_tx);
        let handler = WebSocketHandler::new(
            connection,
            publisher.clone(),
            heartbeat_interval,
            client_timeout,
        );

        (
            Self {
                ctx,

                sessions: HashMap::new(),
                user_sessions: HashMap::new(),
                subscribers: HashMap::new(),

                connection_rx,
                broadcast_rx,
            },
            handler,
            publisher,
        )
    }

    pub fn run(mut self, cancel_token: CancellationToken) -> JoinHandle<()> {
        hb_log::info(Some("💫"), "ApiWebSocketServer: Running component");

        tokio::spawn((|| async move {
            loop {
                select! {
                    _ = cancel_token.cancelled() => {
                        break;
                    }
                    _ = tokio::signal::ctrl_c() => {
                        break;
                    }
                    connection = self.connection_rx.recv() => {
                        if let Some(connection) = connection {
                            match connection {
                                Connection::Connect {
                                    user_id,
                                    token_id,
                                    collection_id,
                                    connection_id,
                                    connection_tx,
                                } => self.insert_connection(
                                    user_id,
                                    token_id,
                                    collection_id,
                                    connection_id,
                                    connection_tx,
                                ),
                                Connection::Disconnect(connection_id) => self.disconnect(connection_id),
                            }
                        } else {
                            break;
                        }
                    }
                    broadcast = self.broadcast_rx.recv() => {
                        if let Some((resource, message)) = broadcast {
                            let _ = self.broadcast(resource, message).await;
                        } else {
                            break;
                        }
                    }
                }
            }

            hb_log::info(None, "ApiWebSocketServer: Shutting down component");
        })())
    }

    fn insert_connection(
        &mut self,
        user_id: Option<UserId>,
        token_id: TokenId,
        collection_id: CollectionId,
        connection_id: ConnectionId,
        connection_tx: mpsc::UnboundedSender<Message>,
    ) {
        self.sessions.insert(connection_id, connection_tx);
        self.user_sessions
            .insert(connection_id, (user_id, token_id));
        self.subscribers
            .entry(collection_id)
            .or_default()
            .insert(connection_id);
    }

    fn disconnect(&mut self, connection_id: ConnectionId) {
        if self.sessions.remove(&connection_id).is_some() {
            for (_, connection_ids) in &mut self.subscribers {
                connection_ids.remove(&connection_id);
                break;
            }
        }
    }

    async fn broadcast(&self, collection_id: CollectionId, message: Message) -> Result<()> {
        if let Some(connection_ids) = self.subscribers.get(&collection_id) {
            for connection_id in connection_ids {
                if let Some((user_id, token_id)) = self.user_sessions.get(connection_id) {
                    if let Some(user_id) = user_id {
                        let token_data = TokenDao::db_select(self.ctx.dao().db(), token_id).await?;
                        if let Some(permission) = token_data
                            .is_allow_find_many_records(self.ctx.dao().db(), &collection_id)
                            .await
                        {
                            if permission == CollectionPermission::SelfMade
                                && message.created_by != *user_id
                            {
                                continue;
                            }
                        } else {
                            continue;
                        }
                    }

                    if let Some(connection_tx) = self.sessions.get(connection_id) {
                        let _ = connection_tx.send(message.clone());
                    }
                }
            }
        }

        Ok(())
    }
}
