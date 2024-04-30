use anyhow::Result;
use hb_api_websocket::{
    broadcaster::WebSocketBroadcaster,
    message::{Message, MessageKind, Target},
};
use serde::Serialize;
use uuid::Uuid;

pub fn websocket_broadcast<T>(
    broadcaster: &WebSocketBroadcaster,
    target: Target,
    created_by: Option<Uuid>,
    kind: MessageKind,
    data: T,
) -> Result<()>
where
    T: Serialize,
{
    let data = serde_json::to_value(data)?;
    broadcaster.broadcast(Message::new(target, created_by, kind, data))
}
