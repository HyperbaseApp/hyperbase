use std::net::SocketAddr;

use ahash::{HashSet, HashSetExt};
use rand::{prelude::SliceRandom, Rng};

use crate::peer::Peer;

#[derive(Clone)]
pub struct View {
    local_address: SocketAddr,
    peers: Vec<Peer>,
}

impl View {
    pub fn new(local_address: SocketAddr, peers: Vec<Peer>) -> Self {
        Self {
            local_address,
            peers,
        }
    }

    pub fn select_peer(&self) -> Option<Peer> {
        if self.peers.is_empty() {
            None
        } else {
            let peer_idx = rand::thread_rng().gen_range(0..self.peers.len());
            Some(self.peers[peer_idx])
        }
    }

    pub fn permute(&mut self) {
        self.peers.shuffle(&mut rand::thread_rng())
    }

    pub fn move_oldest_to_end(&mut self) {
        self.peers.sort_by(|a, b| a.age().cmp(b.age()));
    }

    pub fn head(&self, c: &usize) -> Vec<Peer> {
        let count = std::cmp::min(c / 2 - 1, self.peers.len());
        let mut head = Vec::with_capacity(count);
        for i in 0..count {
            head.push(self.peers[i]);
        }
        head
    }

    pub fn increase_age(&mut self) {
        for peer in &mut self.peers {
            peer.increment_age();
        }
    }

    pub fn select(&mut self, c: &usize, h: &usize, s: &usize, peers: &Vec<Peer>) {
        self.append_peers(peers);
        self.remove_duplicates();
        self.remove_old_items(c, h);
        self.remove_head(c, s);
        self.remove_at_random(c);
    }

    fn append_peers(&mut self, peers: &Vec<Peer>) {
        for peer in peers {
            if *peer.address() != self.local_address {
                self.peers.push(*peer);
            }
        }
    }

    fn remove_duplicates(&mut self) {
        let mut unique_peers = HashSet::<Peer>::with_capacity(self.peers.len());
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