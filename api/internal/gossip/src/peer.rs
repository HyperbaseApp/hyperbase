use std::{
    hash::{Hash, Hasher},
    net::SocketAddr,
};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Copy)]
pub struct PeerDescriptor {
    address: SocketAddr,
    age: u16,
}

impl PeerDescriptor {
    pub fn new(address: SocketAddr) -> PeerDescriptor {
        PeerDescriptor { address, age: 0 }
    }

    pub fn increment_age(&mut self) {
        if self.age < u16::MAX {
            self.age += 1;
        }
    }

    pub fn age(&self) -> &u16 {
        &self.age
    }

    pub fn address(&self) -> &SocketAddr {
        &self.address
    }
}

impl Eq for PeerDescriptor {}

impl PartialEq for PeerDescriptor {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address
    }
}

impl Hash for PeerDescriptor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.address.hash(state);
    }
}
