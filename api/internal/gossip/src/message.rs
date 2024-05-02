use std::net::SocketAddr;

use ahash::HashMap;
use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{handler::MessageHandler, peer::PeerDescriptor};

pub mod database_action;

#[derive(Deserialize, Serialize)]
pub enum Message {
    Sampling {
        kind: MessageKind,
        value: Option<Vec<PeerDescriptor>>,
    },
    Header {
        kind: MessageKind,
        value: Vec<String>,
    },
    Content {
        kind: MessageKind,
        value: HashMap<String, Vec<u8>>,
    },
}

impl Message {
    pub fn handle(self, sender_address: &SocketAddr, handler: MessageHandler) {
        match self {
            Message::Sampling { kind, value } => {
                if let Err(err) = handler.send_sampling(*sender_address, kind, value) {
                    hb_log::error(
                        None,
                        &format!("[ApiInternalGossip] Error sending sample to its handler: {err}"),
                    )
                }
            }
            Message::Header { kind, value } => todo!(),
            Message::Content { kind, value } => todo!(),
        }
    }

    pub fn as_bytes(&self) -> Result<Vec<u8>>
    where
        Self: Serialize,
    {
        let bytes = rmp_serde::to_vec(self)?;
        Ok(bytes)
    }

    pub fn from_bytes<'a>(bytes: &'a [u8]) -> Result<Self>
    where
        Self: Deserialize<'a>,
    {
        let msg = rmp_serde::from_slice(bytes)?;
        Ok(msg)
    }
}

#[derive(Deserialize, Serialize, PartialEq)]
pub enum MessageKind {
    Request,
    Response,
}
