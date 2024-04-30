use anyhow::Result;
use hb_api_websocket::{
    handler::WebSocketHandler,
    message::{Message, MessageKind, Target},
};
use serde::Serialize;
use uuid::Uuid;

pub fn websocket_broadcast<T>(
    handler: &WebSocketHandler,
    target: Target,
    created_by: Option<Uuid>,
    kind: MessageKind,
    data: T,
) -> Result<()>
where
    T: Serialize,
{
    let data = serde_json::to_value(data)?;
    handler.broadcast(Message::new(target, created_by, kind, data))
}
