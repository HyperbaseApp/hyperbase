use std::net::SocketAddr;

use anyhow::Result;
use uuid::Uuid;

use crate::{
    message::{
        content::{ContentChannelSender, ContentMessage},
        header::{HeaderChannelSender, HeaderMessage},
        MessageKind,
    },
    peer::{Peer, PeerSamplingSender},
};

#[derive(Clone)]
pub struct MessageHandler {
    peer_sampling_tx: PeerSamplingSender,
    header_messaging_tx: HeaderChannelSender,
    content_messaging_tx: ContentChannelSender,
}

impl MessageHandler {
    pub fn new(
        peer_sampling_tx: PeerSamplingSender,
        header_messaging_tx: HeaderChannelSender,
        content_messaging_tx: ContentChannelSender,
    ) -> Self {
        Self {
            peer_sampling_tx,
            header_messaging_tx,
            content_messaging_tx,
        }
    }

    pub fn send_sampling(
        &self,
        sender: SocketAddr,
        kind: MessageKind,
        peers: Option<Vec<Peer>>,
    ) -> Result<()> {
        Ok(self.peer_sampling_tx.send((sender, kind, peers))?)
    }

    pub fn send_header(
        &self,
        sender: SocketAddr,
        from: Uuid,
        to: Uuid,
        header: HeaderMessage,
    ) -> Result<()> {
        Ok(self.header_messaging_tx.send((sender, from, to, header))?)
    }

    pub fn send_content(
        &self,
        sender: SocketAddr,
        from: Uuid,
        to: Uuid,
        content: ContentMessage,
    ) -> Result<()> {
        Ok(self
            .content_messaging_tx
            .send((sender, from, to, content))?)
    }
}
