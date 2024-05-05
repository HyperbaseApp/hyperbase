use std::net::SocketAddr;

use peer::PeerDescriptor;
use server::GossipServer;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::{
    config::peer_sampling::PeerSamplingConfig,
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

pub struct ApiInternalGossip {
    address: SocketAddr,
    peers: Vec<PeerDescriptor>,
}

impl ApiInternalGossip {
    pub fn new(host: &str, port: &u16, peers: &Option<Vec<SocketAddr>>) -> Self {
        let address = format!("{host}:{port}").parse().unwrap();

        Self {
            address,
            peers: match peers {
                Some(peers) => peers.iter().map(|p| PeerDescriptor::new(*p)).collect(),
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
            let (peer_sampling_service, peer_sampling_tx) =
                PeerSamplingService::new(self.address, PeerSamplingConfig::default(), self.peers);

            let (database_messaging_service, database_messaging_tx) =
                DatabaseMessagingService::new();

            let server = GossipServer::new(
                self.address,
                MessageHandler::new(peer_sampling_tx, database_messaging_tx),
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
                // _ = database_messaging_service => {}
            }

            hb_log::info(None, "[ApiInternalGossip] Shutting down component");
            server_handle.stop().await;
        })())
    }
}
