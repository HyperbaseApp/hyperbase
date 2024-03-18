use ahash::HashMap;
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct Payload {
    project_id: Uuid,

    token_id: Uuid,
    token: String,
    device: DevicePayload,

    collection_id: Uuid,
    data: Option<HashMap<String, Value>>,
}

impl Payload {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn token_id(&self) -> &Uuid {
        &self.token_id
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn device(&self) -> &DevicePayload {
        &self.device
    }

    pub fn collection_id(&self) -> &Uuid {
        &self.collection_id
    }

    pub fn data(&self) -> &Option<HashMap<String, Value>> {
        &self.data
    }
}

#[derive(Deserialize)]
pub struct DevicePayload {
    collection_id: Uuid,
    id: Uuid,
}

impl DevicePayload {
    pub fn collection_id(&self) -> &Uuid {
        &self.collection_id
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }
}
