use serde::Serialize;
use uuid::Uuid;

use crate::UserId;

#[derive(Serialize, Clone)]
pub struct Message {
    #[serde(skip_serializing)]
    pub target: Target,
    #[serde(skip_serializing)]
    pub created_by: Option<UserId>,

    kind: MessageKind,
    data: serde_json::Value,
}

impl Message {
    pub fn new(
        target: Target,
        created_by: Option<UserId>,
        kind: MessageKind,
        data: serde_json::Value,
    ) -> Self {
        Self {
            target,
            created_by,
            kind,
            data,
        }
    }
}

#[derive(Eq, Hash, PartialEq, Clone)]
pub enum Target {
    Collection(Uuid),
    Log,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum MessageKind {
    InsertOne,
    UpdateOne,
    DeleteOne,
}
