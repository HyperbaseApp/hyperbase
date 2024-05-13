use std::net::SocketAddr;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{handler::MessageHandler, peer::Peer};

use self::{content::ContentMessage, header::HeaderMessage};

pub mod content;
pub mod header;

#[derive(Deserialize, Serialize)]

pub struct Message {
    sender: SocketAddr,
    msg: MessageV,
}

impl Message {
    pub fn new(sender: SocketAddr, msg: MessageV) -> Self {
        Self { sender, msg }
    }

    pub fn handle(self, handler: MessageHandler) {
        match self.msg {
            MessageV::Sampling { kind, data } => {
                if let Err(err) = handler.send_sampling(self.sender, kind, data) {
                    hb_log::error(
                        None,
                        &format!("[ApiInternalGossip] Error sending sample message to its handler: {err}"),
                    )
                }
            }
            MessageV::Header { from, to, data } => {
                if let Err(err) = handler.send_header(self.sender, from, to, data) {
                    hb_log::error(
                        None,
                        &format!("[ApiInternalGossip] Error sending header message to its handler: {err}"),
                    )
                }
            }
            MessageV::Content { from, to, data } => {
                if let Err(err) = handler.send_content(self.sender, from, to, data) {
                    hb_log::error(
                    None,
                    &format!("[ApiInternalGossip] Error sending content message to its handler: {err}"),
                )
                }
            }
        }
    }

    pub fn from_bytes<'a>(bytes: &'a [u8]) -> Result<Self>
    where
        Self: Deserialize<'a>,
    {
        Ok(bincode::deserialize(bytes)?)
    }

    pub fn to_vec(&self) -> Result<Vec<u8>> {
        Ok(bincode::serialize(self)?)
    }
}

#[derive(Deserialize, Serialize)]
pub enum MessageV {
    Sampling {
        kind: MessageKind,
        data: Option<Vec<Peer>>,
    },
    Header {
        from: Uuid,
        to: Uuid,
        data: HeaderMessage,
    },
    Content {
        from: Uuid,
        to: Uuid,
        data: ContentMessage,
    },
}

#[derive(Deserialize, Serialize, PartialEq)]
pub enum MessageKind {
    Request,
    Response,
}
