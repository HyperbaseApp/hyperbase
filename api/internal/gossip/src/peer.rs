use std::{
    hash::{Hash, Hasher},
    net::SocketAddr,
};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Copy)]
pub struct Peer {
    address: SocketAddr,
    age: u16,
}

impl Peer {
    pub fn new(address: SocketAddr) -> Peer {
        Peer { address, age: 0 }
    }

    pub fn increment_age(&mut self) {
        if self.age < u16::MAX {
            self.age += 1;
        }
    }

    pub fn address(&self) -> &SocketAddr {
        &self.address
    }

    pub fn age(&self) -> &u16 {
        &self.age
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
