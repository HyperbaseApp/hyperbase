use std::{
    hash::{Hash, Hasher},
    net::SocketAddr,
};

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::message::MessageKind;

type PeerSampling = (SocketAddr, MessageKind, Option<Vec<Peer>>);
pub type PeerSamplingSender = mpsc::UnboundedSender<PeerSampling>;
pub type PeerSamplingReceiver = mpsc::UnboundedReceiver<PeerSampling>;

#[derive(Deserialize, Serialize, Clone, Copy)]
pub struct Peer {
    id: Option<Uuid>,
    address: SocketAddr,
    age: u16,
}

impl Peer {
    pub fn new(id: Option<Uuid>, address: SocketAddr) -> Peer {
        Peer {
            id,
            address,
            age: 0,
        }
    }

    pub fn id(&self) -> &Option<Uuid> {
        &self.id
    }

    pub fn address(&self) -> &SocketAddr {
        &self.address
    }

    pub fn age(&self) -> &u16 {
        &self.age
    }

    pub fn increment_age(&mut self) {
        if self.age < u16::MAX {
            self.age += 1;
        }
    }
}

impl Eq for Peer {}

impl PartialEq for Peer {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address
    }
}

impl Hash for Peer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.address.hash(state);
    }
}
