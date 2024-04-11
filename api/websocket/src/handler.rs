use std::time::{Duration, Instant};

use anyhow::Result;
use futures::StreamExt;
use tokio::{select, sync::mpsc, time::interval};
use uuid::Uuid;

use crate::{
    broadcaster::WebSocketBroadcaster,
    connection::WebSocketConnection,
    server::{Message, Target, UserSession},
};

#[derive(Clone)]
pub struct WebSocketHandler {
    connection: WebSocketConnection,
    broadcaster: WebSocketBroadcaster,

    heartbeat_interval: Duration,
    client_timeout: Duration,
}

impl WebSocketHandler {
    pub fn new(
        connection: WebSocketConnection,
        broadcaster: WebSocketBroadcaster,
        heartbeat_interval: &Duration,
        client_timeout: &Duration,
    ) -> Self {
        Self {
            connection,
            broadcaster,
            heartbeat_interval: *heartbeat_interval,
            client_timeout: *client_timeout,
        }
    }

    pub async fn connection(
        self,
        user_session: UserSession,
        target: Target,
        mut session: actix_ws_ng::Session,
        mut msg_stream: actix_ws_ng::MessageStream,
    ) -> Result<()> {
        let connection_id = Uuid::now_v7();

        let (connection_tx, mut connection_rx) = mpsc::unbounded_channel();

        self.connection
            .connect(user_session, target, connection_id, connection_tx)?;

        let mut last_heartbeat = Instant::now();
        let mut interval = interval(self.heartbeat_interval);

        let close_reason = loop {
            select! {
                _ = interval.tick() => {
                    if Instant::now().duration_since(last_heartbeat) > self.client_timeout {
                        hb_log::info(
                            None,
                            &format!(
                                "ApiWebSocketServer: Disconnecting connection '{}' because client has not sent hearbeat in over {:?}",
                                connection_id, self.client_timeout
                            ),
                        );
                        break None;
                    }
                    if session.ping(b"").await.is_err() {
                        hb_log::info(
                            None,
                            &format!(
                                "ApiWebSocketServer: Session with connection '{connection_id}' is closed"
                            ),
                        );
                        break None;
                    }
                }
                msg = msg_stream.next() => {
                    if let Some(Ok(msg)) = msg{
                        match msg {
                            actix_ws_ng::Message::Ping(bytes) => {
                                last_heartbeat = Instant::now();
                                let _ = session.pong(&bytes).await;
                            }
                            actix_ws_ng::Message::Pong(_) => last_heartbeat = Instant::now(),
                            actix_ws_ng::Message::Close(reason) => break reason,
                            actix_ws_ng::Message::Text(_) | actix_ws_ng::Message::Binary(_) => (),
                            _ => break None,
                        }
                    } else if let Some(Err(err)) = msg {
                        hb_log::warn(
                            None,
                            format!(
                                "ApiWebSocketServer: Message stream of connection '{connection_id}' throws error {err}",
                            ),
                        );
                        break None;
                    } else {
                        break None;
                    }
                }
                Some(msg) = connection_rx.recv() => {
                    let msg = match serde_json::to_string(&msg) {
                        Ok(msg) => msg,
                        Err(err) => {
                            hb_log::error(
                                None,
                                &format!(
                                    "ApiWebSocketServer: Error when serializing message: {err}",
                                ),
                            );
                            continue;
                        }
                    };
                    if session.text(msg).await.is_err() {
                        hb_log::info(
                            None,
                            &format!(
                                "ApiWebSocketServer: Session with connection '{connection_id}' is closed"
                            ),
                        );
                        break None;
                    }
                }
            }
        };

        self.connection.disconnect(connection_id)?;

        session.close(close_reason).await?;

        Ok(())
    }

    pub fn broadcast(&self, message: Message) -> Result<()> {
        self.broadcaster.broadcast(message)
    }
}
