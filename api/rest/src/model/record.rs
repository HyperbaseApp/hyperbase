use ahash::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct InsertOneRecordReqPath {
    project_id: Uuid,
    collection_id: Uuid,
}

impl InsertOneRecordReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn collection_id(&self) -> &Uuid {
        &self.collection_id
    }
}

pub type InsertOneRecordReqJson = HashMap<String, Value>;

#[derive(Deserialize)]
pub struct FindOneRecordReqPath {
    project_id: Uuid,
    collection_id: Uuid,
    record_id: Uuid,
}

impl FindOneRecordReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn collection_id(&self) -> &Uuid {
        &self.collection_id
    }

    pub fn record_id(&self) -> &Uuid {
        &self.record_id
    }
}

#[derive(Deserialize)]
pub struct UpdateOneRecordReqPath {
    project_id: Uuid,
    collection_id: Uuid,
    record_id: Uuid,
}

impl UpdateOneRecordReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

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
    project_id: Uuid,
    collection_id: Uuid,
    record_id: Uuid,
}

impl DeleteOneRecordReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn collection_id(&self) -> &Uuid {
        &self.collection_id
    }

    pub fn record_id(&self) -> &Uuid {
        &self.record_id
    }
}

#[derive(Deserialize)]
pub struct FindManyRecordReqPath {
    project_id: Uuid,
    collection_id: Uuid,
}

impl FindManyRecordReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn collection_id(&self) -> &Uuid {
        &self.collection_id
    }
}

#[derive(Serialize)]
pub struct RecordResJson {
    #[serde(flatten)]
    data: HashMap<String, Value>,
}

impl RecordResJson {
    pub fn new(data: &HashMap<String, Value>) -> Self {
        Self { data: data.clone() }
    }
}

#[derive(Serialize)]
pub struct DeleteRecordResJson {
    id: Uuid,
}

impl DeleteRecordResJson {
    pub fn new(id: &Uuid) -> Self {
        Self { id: *id }
    }
}
