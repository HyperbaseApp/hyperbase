use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct InsertOneRecordReqPath {
    collection_id: Uuid,
}

impl InsertOneRecordReqPath {
    pub fn collection_id(&self) -> &Uuid {
        &self.collection_id
    }
}

pub type InsertOneRecordReqJson = HashMap<String, Value>;

#[derive(Deserialize)]
pub struct FindOneRecordReqPath {
    collection_id: Uuid,
    record_id: Uuid,
}

impl FindOneRecordReqPath {
    pub fn collection_id(&self) -> &Uuid {
        &self.collection_id
    }

    pub fn record_id(&self) -> &Uuid {
        &self.record_id
    }
}

#[derive(Deserialize)]
pub struct UpdateOneRecordReqPath {
    collection_id: Uuid,
    record_id: Uuid,
}

impl UpdateOneRecordReqPath {
    pub fn collection_id(&self) -> &Uuid {
        &self.collection_id
    }

    pub fn record_id(&self) -> &Uuid {
        &self.record_id
    }
}

pub type UpdateOneRecordReqJson = HashMap<String, Value>;

#[derive(Deserialize)]
pub struct DeleteOneRecordReqPath {
    collection_id: Uuid,
    record_id: Uuid,
}

impl DeleteOneRecordReqPath {
    pub fn collection_id(&self) -> &Uuid {
        &self.collection_id
    }

    pub fn record_id(&self) -> &Uuid {
        &self.record_id
    }
}
