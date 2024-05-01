use std::{net::SocketAddr, sync::Arc, time::Duration};

use ahash::{HashSet, HashSetExt};
use rand::{prelude::SliceRandom, Rng};
use tokio::{
    sync::{mpsc, Mutex},
    task::JoinHandle,
};

use crate::{
    client,
    config::peer_sampling::PeerSamplingConfig,
    message::{Message, MessageBody, MessageKind},
    peer::PeerDescriptor,
};

pub struct PeerSamplingService {
    local_address: SocketAddr,
    config: PeerSamplingConfig,
    view: Arc<Mutex<View>>,

    rx: mpsc::UnboundedReceiver<(SocketAddr, MessageKind, Option<Vec<PeerDescriptor>>)>,
}

impl PeerSamplingService {
    pub fn new(
        local_address: SocketAddr,
        config: PeerSamplingConfig,
        peers: Vec<PeerDescriptor>,
    ) -> (
        Self,
        mpsc::UnboundedSender<(SocketAddr, MessageKind, Option<Vec<PeerDescriptor>>)>,
    ) {
        let (tx, rx) = mpsc::unbounded_channel();
        (
            Self {
                local_address,
                config,
                view: Arc::new(Mutex::new(View::new(local_address, peers))),
                rx,
            },
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
        view: Arc<Mutex<View>>,
        mut receiver: mpsc::UnboundedReceiver<(
            SocketAddr,
            MessageKind,
            Option<Vec<PeerDescriptor>>,
        )>,
    ) {
        while let Some((sender_address, kind, peers)) = receiver.recv().await {
            let view = view.clone();
            tokio::spawn((|| async move {
                let mut view = view.lock().await;
                if kind == MessageKind::Request && *config.pull() {
                    let buffer = Self::build_local_view_buffer(&local_address, &config, &mut view);
                    match client::send(
                                &sender_address,
                                Message::new(
                                    local_address,
                                    MessageBody::Sampling {
                                        kind: MessageKind::Response,
                                        value: Some(buffer),
                                    },
                                ),
                            )
                            .await
                            {
                                Ok(written) => hb_log::info(None, &format!("[ApiInternalGossip] Peer sampling with local view response sent successfully to {sender_address} ({written} bytes)")),
                                Err(err) => hb_log::error(None, &format!("[ApiInternalGossip] Peer sampling with local view response failed to send to {sender_address} due to error: {err}")),
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
                    let buffer = Self::build_local_view_buffer(&local_address, &config, &mut view);
                    match client::send(
                            peer.address(),
                            Message::new(
                                local_address,
                                MessageBody::Sampling { kind: MessageKind::Request, value: Some(buffer) },
                            ),
                        )
                        .await
                        {
                            Ok(written) => hb_log::info(None, &format!("[ApiInternalGossip] Peer sampling with local view request sent successfully to {} ({} bytes)", peer.address(), written)),
                            Err(err) => hb_log::error(None, &format!("[ApiInternalGossip] Peer sampling with local view request failed to send to {} due to error: {}", peer.address(), err)),
                        }
                } else {
                    match client::send(
                            peer.address(),
                            Message::new(
                                local_address,
                                MessageBody::Sampling { kind: MessageKind::Request, value: None },
                            ),
                        ).await {
                            Ok(written) => hb_log::info(None, &format!("[ApiInternalGossip] Peer sampling with empty view request sent successfully to {} ({} bytes)", peer.address(), written)),
                            Err(err) => hb_log::error(None, &format!("[ApiInternalGossip] Peer sampling with empty view request failed to send to {} due to error: {}", peer.address(), err)),
                        }
                }
                view.increase_age();
            } else {
                hb_log::warn(None, "[ApiInternalGossip] No peer found for peer sampling")
            }

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

    fn build_local_view_buffer(
        local_address: &SocketAddr,
        config: &PeerSamplingConfig,
        view: &mut View,
    ) -> Vec<PeerDescriptor> {
        let mut buffer = vec![PeerDescriptor::new(*local_address)];
        view.permute();
        view.move_oldest_to_end();
        let mut view_head = view.head(config.view_size());
        buffer.append(&mut view_head);
        buffer
    }
}

#[derive(Clone)]
pub struct View {
    local_address: SocketAddr,
    peers: Vec<PeerDescriptor>,
}

impl View {
    fn new(local_address: SocketAddr, peers: Vec<PeerDescriptor>) -> Self {
        Self {
            local_address,
            peers,
        }
    }

    fn select_peer(&self) -> Option<PeerDescriptor> {
        if self.peers.is_empty() {
            None
        } else {
            let peer_idx = rand::thread_rng().gen_range(0..self.peers.len());
            Some(self.peers[peer_idx])
        }
    }

    fn permute(&mut self) {
        self.peers.shuffle(&mut rand::thread_rng())
    }

    fn move_oldest_to_end(&mut self) {
        self.peers.sort_by(|a, b| a.age().cmp(b.age()));
    }

    fn head(&self, c: &usize) -> Vec<PeerDescriptor> {
        let count = std::cmp::min(c / 2 - 1, self.peers.len());
        let mut head = Vec::with_capacity(count);
        for i in 0..count {
            head.push(self.peers[i]);
        }
        head
    }

    fn increase_age(&mut self) {
        for peer in &mut self.peers {
            peer.increment_age();
        }
    }

    fn select(&mut self, c: &usize, h: &usize, s: &usize, peers: &Vec<PeerDescriptor>) {
        self.append_peers(peers);
        self.remove_duplicates();
        self.remove_old_items(c, h);
        self.remove_head(c, s);
        self.remove_at_random(c);
    }

    fn append_peers(&mut self, peers: &Vec<PeerDescriptor>) {
        for peer in peers {
            if *peer.address() != self.local_address {
                self.peers.push(*peer);
            }
        }
    }

    fn remove_duplicates(&mut self) {
        let mut unique_peers = HashSet::<PeerDescriptor>::with_capacity(self.peers.len());
        for peer in &self.peers {
            match unique_peers.get(peer) {
                Some(entry) => {
                    if entry.age() > peer.age() {
                        unique_peers.replace(*peer);
                    }
                }
                None => {
                    unique_peers.insert(*peer);
                }
            }
        }
        let new_peers = Vec::from_iter(unique_peers);
        self.peers = new_peers;
    }

    fn remove_old_items(&mut self, c: &usize, h: &usize) {
        let removal_count = std::cmp::min(
            *h,
            if self.peers.len() > *c {
                self.peers.len() - *c
            } else {
                0
            },
        );
        if removal_count > 0 {
            self.move_oldest_to_end();
            self.peers.truncate(self.peers.len() - removal_count);
        }
    }

    fn remove_head(&mut self, c: &usize, s: &usize) {
        let removal_count = std::cmp::min(
            *s,
            if self.peers.len() > *c {
                self.peers.len() - *c
            } else {
                0
            },
        );
        if removal_count > 0 {
            self.peers.drain(0..removal_count);
        }
    }

    fn remove_at_random(&mut self, c: &usize) {
        while self.peers.len() > *c {
            let remove_index = rand::thread_rng().gen_range(0..self.peers.len());
            self.peers.remove(remove_index);
        }
    }
}
