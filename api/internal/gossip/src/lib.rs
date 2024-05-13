use std::{
    net::{SocketAddr, ToSocketAddrs},
    sync::Arc,
};

use anyhow::Result;
use hb_dao::{change::ChangeDao, local_info::LocalInfoDao, Db};
use message::content::{ContentChangeModel, ContentChannelSender, ContentMessage};
use peer::Peer;
use server::{GossipServer, GossipServerRunner};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::{
    config::{database_messaging::DatabaseMessagingConfig, peer_sampling::PeerSamplingConfig},
    handler::MessageHandler,
    service::{database_messaging::DatabaseMessagingService, peer_sampling::PeerSamplingService},
};

mod client;
mod config;
mod handler;
pub mod message;
mod peer;
mod server;
mod service;
pub mod view;

pub struct ApiInternalGossip {
    peer_sampling_service: PeerSamplingService,
    database_messaging_service: DatabaseMessagingService,
    server: GossipServerRunner,
}

impl ApiInternalGossip {
    pub async fn new(
        host: &str,
        port: &u16,
        db: Arc<Db>,
        peers: &Option<Vec<String>>,
        view_size: &usize,
        actions_size: &i32,
    ) -> (Self, ContentChannelSender) {
        let local_id = match LocalInfoDao::db_select(&db).await {
            Ok(data) => *data.id(),
            Err(_) => {
                let local_info_data = LocalInfoDao::new();
                let _ = local_info_data.db_insert(&db).await;
                let local_info_data = LocalInfoDao::db_select(&db).await.unwrap();
                *local_info_data.id()
            }
        };

        let local_address = format!("{host}:{port}")
            .to_socket_addrs()
            .unwrap()
            .next()
            .unwrap();

        let peers = match peers {
            Some(peers) => peers
                .iter()
                .map(|p| Peer::new(None, p.to_socket_addrs().unwrap().next().unwrap()))
                .collect(),
            None => Vec::new(),
        };

        let (peer_sampling_service, view, peer_sampling_tx) = PeerSamplingService::new(
            local_id,
            local_address,
            PeerSamplingConfig::new(view_size),
            db.clone(),
            peers,
        );

        let (database_messaging_service, header_messaging_tx, content_messaging_tx) =
            DatabaseMessagingService::new(
                local_id,
                local_address,
                DatabaseMessagingConfig::new(actions_size),
                db,
                view,
            );

        let server = GossipServer::new(
            local_address,
            MessageHandler::new(
                peer_sampling_tx,
                header_messaging_tx,
                content_messaging_tx.clone(),
            ),
        )
        .run();

        (
            Self {
                peer_sampling_service,
                database_messaging_service,
                server,
            },
            content_messaging_tx,
        )
    }

    pub fn run_none() -> JoinHandle<()> {
        hb_log::info(Some("⏩"), "[ApiInternalGossip] Skipping component");

        tokio::spawn((|| async {})())
    }

    pub fn run(self, cancel_token: CancellationToken) -> JoinHandle<()> {
        hb_log::info(Some("💫"), "[ApiInternalGossip] Running component");

        tokio::spawn((|| async move {
            let server_handle = self.server.handle();

            let peer_sampling_service = self.peer_sampling_service.run();
            let database_messaging_service = self.database_messaging_service.run();

            tokio::select! {
                _ = cancel_token.cancelled() => {}
                s = self.server => {
                    if let Err(err) = s {
                        hb_log::panic(None, &format!("[ApiInternalGossip] Gossip server error: {err}"));
                    }
                }
                _ = peer_sampling_service => {}
                _ = database_messaging_service => {}
            }

            hb_log::info(None, "[ApiInternalGossip] Shutting down component");
            server_handle.stop().await;
        })())
    }
}

#[derive(Clone)]
pub struct InternalBroadcast {
    tx: ContentChannelSender,
    db: Arc<Db>,
    local_id: Uuid,
    local_address: SocketAddr,
}

impl InternalBroadcast {
    pub async fn new(tx: ContentChannelSender, db: Arc<Db>, host: &str, port: &u16) -> Self {
        let local_info_data = LocalInfoDao::db_select(&db).await.unwrap();

        let local_address = format!("{host}:{port}")
            .to_socket_addrs()
            .unwrap()
            .next()
            .unwrap();

        Self {
            tx,
            db,
            local_id: *local_info_data.id(),
            local_address,
        }
    }

    pub async fn broadcast(&self, change_data: &ChangeDao) -> Result<()> {
        let change_data = ContentChangeModel::from_change_dao(&self.db, change_data).await?;
        Ok(self.tx.send((
            self.local_address,
            self.local_id,
            self.local_id,
            ContentMessage::Broadcast { change_data },
        ))?)
    }
}
