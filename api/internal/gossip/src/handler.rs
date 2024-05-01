use std::net::SocketAddr;

use anyhow::Result;
use tokio::sync::mpsc;

use crate::{message::MessageKind, peer::PeerDescriptor};

#[derive(Clone)]
pub struct MessageHandler {
    peer_sampling_tx: mpsc::UnboundedSender<(SocketAddr, MessageKind, Option<Vec<PeerDescriptor>>)>,
}

impl MessageHandler {
    pub fn new(
        peer_sampling_tx: mpsc::UnboundedSender<(
            SocketAddr,
            MessageKind,
            Option<Vec<PeerDescriptor>>,
        )>,
    ) -> Self {
        Self { peer_sampling_tx }
    }

    pub fn send_sampling(
        &self,
        sender: SocketAddr,
        kind: MessageKind,
        peers: Option<Vec<PeerDescriptor>>,
    ) -> Result<()> {
        self.peer_sampling_tx.send((sender, kind, peers))?;
        Ok(())
    }
}
