use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct InsertOneRecordPath {
    collection_id: Uuid,
}

impl InsertOneRecordPath {
    pub fn collection_id(&self) -> &Uuid {
        &self.collection_id
    }
}

pub type InsertOneRecordJson = HashMap<String, Value>;

#[derive(Deserialize)]
pub struct FindOneRecordPath {
    collection_id: Uuid,
    record_id: Uuid,
}

impl FindOneRecordPath {
    pub fn collection_id(&self) -> &Uuid {
        &self.collection_id
    }

    pub fn record_id(&self) -> &Uuid {
        &self.record_id
    }
}

#[derive(Deserialize)]
pub struct UpdateOneRecordPath {
    collection_id: Uuid,
    record_id: Uuid,
}

impl UpdateOneRecordPath {
    pub fn collection_id(&self) -> &Uuid {
        &self.collection_id
    }

    pub fn record_id(&self) -> &Uuid {
        &self.record_id
    }
}

pub type UpdateOneRecordJson = HashMap<String, Value>;

#[derive(Deserialize)]
pub struct DeleteOneRecordPath {
    collection_id: Uuid,
    record_id: Uuid,
}

impl DeleteOneRecordPath {
    pub fn collection_id(&self) -> &Uuid {
        &self.collection_id
    }

    pub fn record_id(&self) -> &Uuid {
        &self.record_id
    }
}
