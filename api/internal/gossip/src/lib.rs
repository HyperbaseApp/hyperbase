use std::{net::SocketAddr, sync::Arc};

use hb_dao::{local_info::LocalInfoDao, Db};
use peer::Peer;
use server::GossipServer;
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
mod message;
mod peer;
mod server;
mod service;
mod view;

pub struct ApiInternalGossip {
    local_id: Uuid,
    local_address: SocketAddr,
    db: Arc<Db>,
    peers: Vec<Peer>,
}

impl ApiInternalGossip {
    pub async fn new(host: &str, port: &u16, db: Arc<Db>, peers: &Option<Vec<SocketAddr>>) -> Self {
        let local_id = match LocalInfoDao::db_select(&db).await {
            Ok(data) => *data.id(),
            Err(_) => {
                let local_info_data = LocalInfoDao::new();
                local_info_data.db_insert(&db).await.unwrap();
                *local_info_data.id()
            }
        };

        let local_address = format!("{host}:{port}").parse().unwrap();

        Self {
            local_id,
            local_address,
            db,
            peers: match peers {
                Some(peers) => peers.iter().map(|p| Peer::new(None, *p)).collect(),
                None => Vec::new(),
            },
        }
    }

    pub fn run_none() -> JoinHandle<()> {
        hb_log::info(Some("⏩"), "[ApiInternalGossip] Skipping component");

        tokio::spawn((|| async {})())
    }

    pub fn run(self, cancel_token: CancellationToken) -> JoinHandle<()> {
        hb_log::info(Some("💫"), "[ApiInternalGossip] Running component");

        tokio::spawn((|| async move {
            let (peer_sampling_service, view, peer_sampling_tx) = PeerSamplingService::new(
                self.local_id,
                self.local_address,
                PeerSamplingConfig::default(),
                self.peers,
            );

            let (database_messaging_service, header_messaging_tx, content_messaging_tx) =
                DatabaseMessagingService::new(
                    self.local_id,
                    self.local_address,
                    DatabaseMessagingConfig::default(),
                    self.db,
                    view,
                );

            let server = GossipServer::new(
                self.local_address,
                MessageHandler::new(peer_sampling_tx, header_messaging_tx, content_messaging_tx),
            )
            .run();
            let server_handle = server.handle();

            let peer_sampling_service = peer_sampling_service.run();
            let database_messaging_service = database_messaging_service.run();

            tokio::select! {
                _ = cancel_token.cancelled() => {}
                s = server => {
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
