use std::{net::SocketAddr, sync::Arc, time::Duration};

use chrono::DateTime;
use hb_dao::{remote_sync::RemoteSyncDao, Db};
use rand::Rng;
use tokio::{
    sync::{mpsc, Mutex},
    task::JoinHandle,
};
use uuid::Uuid;

use crate::{
    client,
    config::peer_sampling::PeerSamplingConfig,
    message::{Message, MessageKind, MessageV},
    peer::{Peer, PeerSamplingReceiver, PeerSamplingSender},
    view::View,
};

pub struct PeerSamplingService {
    local_address: SocketAddr,
    config: PeerSamplingConfig,
    db: Arc<Db>,
    view: Arc<Mutex<View>>,

    rx: PeerSamplingReceiver,
}

impl PeerSamplingService {
    pub fn new(
        local_id: Uuid,
        local_address: SocketAddr,
        config: PeerSamplingConfig,
        db: Arc<Db>,
        peers: Vec<Peer>,
    ) -> (Self, Arc<Mutex<View>>, PeerSamplingSender) {
        let (tx, rx) = mpsc::unbounded_channel();
        let view = Arc::new(Mutex::new(View::new(local_id, local_address, peers)));
        (
            Self {
                local_address,
                config,
                db,
                view: view.clone(),
                rx,
            },
            view,
            tx,
        )
    }

    pub fn run(self) -> JoinHandle<()> {
        hb_log::info(
            Some("🧩"),
            "[ApiInternalGossip] Running peer sampling service",
        );

        tokio::spawn((|| async move {
            tokio::join!(
                Self::run_receiver_task(
                    self.local_address,
                    self.config,
                    self.db,
                    self.view.clone(),
                    self.rx
                ),
                Self::run_sender_task(self.local_address, self.config, self.view)
            );
        })())
    }

    async fn run_receiver_task(
        local_address: SocketAddr,
        config: PeerSamplingConfig,
        db: Arc<Db>,
        view: Arc<Mutex<View>>,
        mut receiver: PeerSamplingReceiver,
    ) {
        while let Some((sender_address, kind, peers)) = receiver.recv().await {
            let view = view.clone();
            let db = db.clone();
            tokio::spawn((|| async move {
                let mut view = view.lock().await;
                if kind == MessageKind::Request && *config.pull() {
                    let buffer = Self::build_local_view_buffer(&config, &mut view);
                    match client::send(
                                &sender_address,
                                Message::new(local_address, MessageV::Sampling { kind: MessageKind::Response, data: Some(buffer) }),
                            )
                            .await
                            {
                                Ok(written) => hb_log::info(None, &format!("[ApiInternalGossip] Peer sampling with local view response sent successfully to {sender_address} ({written} bytes)")),
                                Err(err) => hb_log::warn(None, &format!("[ApiInternalGossip] Peer sampling with local view response failed to send to {sender_address} due to error: {err}")),
                        }
                }

                if let Some(peers) = peers {
                    if peers.len() > 0 {
                        view.select(
                            config.view_size(),
                            config.healing_factor(),
                            config.swapping_factor(),
                            &peers,
                        );
                        let peers = view.peers();
                        for peer in peers {
                            if let Some(peer_id) = peer.id() {
                                let remotes = match RemoteSyncDao::db_select_many_by_address(
                                    &db,
                                    peer.address(),
                                )
                                .await
                                {
                                    Ok(data) => data,
                                    Err(err) => {
                                        hb_log::error(None, &format!("[ApiInternalGossip] Failed to select remotes data by address: {err}"));
                                        return;
                                    }
                                };
                                let mut exists = false;
                                for remote in &remotes {
                                    if remote.remote_id() == peer_id {
                                        exists = true;
                                        break;
                                    }
                                }
                                if !exists {
                                    let remote_data = RemoteSyncDao::new(
                                        peer_id,
                                        peer.address(),
                                        &DateTime::from_timestamp_millis(1000).unwrap(),
                                        &Uuid::nil(),
                                    );
                                    if let Err(err) = remote_data.db_insert_or_ignore(&db).await {
                                        hb_log::error(None, &format!("[ApiInternalGossip] Failed to insert new remote data: {err}"));
                                        return;
                                    }
                                }
                            }
                        }
                    } else {
                        hb_log::warn(
                            None,
                            "[ApiInternalGossip] Received a peer sampling with zero peers",
                        )
                    }
                } else {
                    hb_log::warn(
                        None,
                        "[ApiInternalGossip] Received a peer sampling with none peers",
                    )
                }

                view.increase_age();
            })());
        }
    }

    async fn run_sender_task(
        local_address: SocketAddr,
        config: PeerSamplingConfig,
        view: Arc<Mutex<View>>,
    ) {
        loop {
            let mut view = view.lock().await;
            if let Some(peer) = view.select_peer() {
                if *config.push() {
                    let buffer = Self::build_local_view_buffer(&config, &mut view);
                    match client::send(
                            peer.address(),
                            Message::new(local_address, MessageV::Sampling { kind: MessageKind::Request, data: Some(buffer) }),
                        )
                        .await
                        {
                            Ok(written) => hb_log::info(None, &format!("[ApiInternalGossip] Peer sampling with local view request sent successfully to {} ({} bytes)", peer.address(), written)),
                            Err(err) => hb_log::warn(None, &format!("[ApiInternalGossip] Peer sampling with local view request failed to send to {} due to error: {}", peer.address(), err)),
                        }
                } else {
                    match client::send(
                            peer.address(),
                            Message::new(local_address, MessageV::Sampling { kind: MessageKind::Request, data: None }),
                        ).await {
                            Ok(written) => hb_log::info(None, &format!("[ApiInternalGossip] Peer sampling with empty view request sent successfully to {} ({} bytes)", peer.address(), written)),
                            Err(err) => hb_log::warn(None, &format!("[ApiInternalGossip] Peer sampling with empty view request failed to send to {} due to error: {}", peer.address(), err)),
                        }
                }
                view.increase_age();
            } else {
                hb_log::warn(None, "[ApiInternalGossip] No peer found for peer sampling")
            }
            drop(view);

            let sleep_duration_deviation = match config.period_deviation() {
                0 => 0,
                val => rand::thread_rng().gen_range(0..=*val),
            };
            let sleep_duration = config.period() + sleep_duration_deviation;

            hb_log::info(
                None,
                format!(
                    "[ApiInternalGossip] Next peer sampling request is after {sleep_duration} ms"
                ),
            );

            tokio::time::sleep(Duration::from_millis(sleep_duration)).await;
        }
    }

    fn build_local_view_buffer(config: &PeerSamplingConfig, view: &View) -> Vec<Peer> {
        let mut view = view.clone().add_with_local();
        view.permute();
        view.move_oldest_to_end();
        view.head(config.view_size())
    }
}
