use ahash::HashMap;
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

#[derive(Deserialize, Clone)]
pub struct Payload {
    project_id: Uuid,

    token_id: Uuid,
    user: Option<UserPayload>,

    collection_id: Uuid,
    data: HashMap<String, Value>,
}

impl Payload {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn token_id(&self) -> &Uuid {
        &self.token_id
    }

    pub fn user(&self) -> &Option<UserPayload> {
        &self.user
    }

    pub fn collection_id(&self) -> &Uuid {
        &self.collection_id
    }

    pub fn data(&self) -> &HashMap<String, Value> {
        &self.data
    }
}

#[derive(Deserialize, Clone)]
pub struct UserPayload {
    collection_id: Uuid,
    id: Uuid,
}

impl UserPayload {
    pub fn collection_id(&self) -> &Uuid {
        &self.collection_id
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }
}
