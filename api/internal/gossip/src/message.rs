use std::net::SocketAddr;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{handler::MessageHandler, peer::Peer};

use self::{content::ContentMessage, header::HeaderMessage};

pub mod content;
pub mod header;

#[derive(Deserialize, Serialize)]
pub enum Message {
    Sampling {
        kind: MessageKind,
        value: Option<Vec<Peer>>,
    },
    Header {
        from: Uuid,
        to: Uuid,
        value: HeaderMessage,
    },
    Content {
        from: Uuid,
        to: Uuid,
        value: ContentMessage,
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
            Message::Header { from, to, value } => todo!(),
            Message::Content { from, to, value } => todo!(),
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
