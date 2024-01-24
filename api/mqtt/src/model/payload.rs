use ahash::HashMap;
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct Payload {
    token: String,
    method: Method,
    project_id: Uuid,
    collection_id: Uuid,
    data: Option<HashMap<String, Value>>,
}

impl Payload {
    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn method(&self) -> &Method {
        &self.method
    }

    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn collection_id(&self) -> &Uuid {
        &self.collection_id
    }

    pub fn data(&self) -> &Option<HashMap<String, Value>> {
        &self.data
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Method {
    InsertOne,
}
