use std::time::Duration;

use ahash::{HashMap, HashMapExt, HashSet};
use anyhow::Result;
use broadcaster::WebSocketBroadcaster;
use connection::{Connection, WebSocketConnection};
use context::ApiWebSocketCtx;
use handler::WebSocketHandler;
use hb_dao::{collection_rule::CollectionPermission, token::TokenDao};
use message::{Message, Target};
use session::UserSession;
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

mod connection;

pub mod broadcaster;
pub mod context;
pub mod handler;
pub mod message;
pub mod session;

pub type ConnectionId = Uuid;
pub type UserId = Uuid;
pub type TokenId = Uuid;
pub type AdminId = Uuid;

pub struct ApiWebSocketServer {
    ctx: ApiWebSocketCtx,

    sessions: HashMap<ConnectionId, mpsc::UnboundedSender<Message>>,
    user_sessions: HashMap<ConnectionId, UserSession>,
    subscribers: HashMap<Target, HashSet<ConnectionId>>,

    connection_rx: mpsc::UnboundedReceiver<Connection>,
    broadcast_rx: mpsc::UnboundedReceiver<Message>,
}

impl ApiWebSocketServer {
    pub fn new(
        ctx: ApiWebSocketCtx,

        heartbeat_interval: &Duration,
        client_timeout: &Duration,
    ) -> (Self, WebSocketHandler, WebSocketBroadcaster) {
        hb_log::info(Some("⚡"), "[ApiWebSocketServer] Initializing component");

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
        hb_log::info(Some("💫"), "[ApiWebSocketServer] Running component");

        tokio::spawn((|| async move {
            loop {
                tokio::select! {
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
                                    user_session,
                                    target,
                                    connection_id,
                                    connection_tx,
                                } => self.insert_connection(
                                    user_session,
                                    target,
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
                        if let Some(message) = broadcast {
                            let _ = self.broadcast(message).await;
                        } else {
                            break;
                        }
                    }
                }
            }

            hb_log::info(None, "[ApiWebSocketServer] Shutting down component");
        })())
    }

    fn insert_connection(
        &mut self,
        user_session: UserSession,
        target: Target,
        connection_id: ConnectionId,
        connection_tx: mpsc::UnboundedSender<Message>,
    ) {
        self.sessions.insert(connection_id, connection_tx);
        self.user_sessions.insert(connection_id, user_session);
        self.subscribers
            .entry(target)
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

    async fn broadcast(&self, message: Message) -> Result<()> {
        if let Some(connection_ids) = self.subscribers.get(&message.target) {
            for connection_id in connection_ids {
                if let Some(user_session) = self.user_sessions.get(connection_id) {
                    if let UserSession::Token(token_id, user_id) = user_session {
                        if message.created_by.is_none() {
                            continue;
                        }
                        if let Target::Collection(collection_id) = &message.target {
                            let token_data = TokenDao::db_select(self.ctx.db(), token_id).await?;
                            if let Some(permission) = token_data
                                .is_allow_find_many_records(self.ctx.db(), collection_id)
                                .await
                            {
                                if permission == CollectionPermission::None {
                                    continue;
                                }
                                if permission == CollectionPermission::SelfMade
                                    && &message.created_by != user_id
                                {
                                    continue;
                                }
                            } else {
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
