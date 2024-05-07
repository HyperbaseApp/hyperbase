use std::net::SocketAddr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use uuid::Uuid;

type HeaderChannel = (SocketAddr, Uuid, Uuid, HeaderMessage);
pub type HeaderChannelSender = mpsc::UnboundedSender<HeaderChannel>;
pub type HeaderChannelReceiver = mpsc::UnboundedReceiver<HeaderChannel>;

#[derive(Deserialize, Serialize)]
pub enum HeaderMessage {
    Request {
        from_time: DateTime<Utc>,
        last_change_id: Uuid,
    },
    Response {
        change_ids: Vec<Uuid>,
    },
}
