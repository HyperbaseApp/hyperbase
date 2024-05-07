use std::net::SocketAddr;

use anyhow::Result;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    message::{content::ContentMessage, header::HeaderMessage, MessageKind},
    peer::Peer,
};

#[derive(Clone)]
pub struct MessageHandler {
    peer_sampling_tx: mpsc::UnboundedSender<(SocketAddr, MessageKind, Option<Vec<Peer>>)>,
    header_messaging_tx: mpsc::UnboundedSender<(SocketAddr, Uuid, Uuid, HeaderMessage)>,
    content_messaging_tx: mpsc::UnboundedSender<(SocketAddr, Uuid, Uuid, ContentMessage)>,
}

impl MessageHandler {
    pub fn new(
        peer_sampling_tx: mpsc::UnboundedSender<(SocketAddr, MessageKind, Option<Vec<Peer>>)>,
        header_messaging_tx: mpsc::UnboundedSender<(SocketAddr, Uuid, Uuid, HeaderMessage)>,
        content_messaging_tx: mpsc::UnboundedSender<(SocketAddr, Uuid, Uuid, ContentMessage)>,
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
