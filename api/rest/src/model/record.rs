use ahash::HashMap;
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime};
use hb_dao::record::ColumnValue;
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

#[derive(Serialize)]
pub struct RecordResJson {
    data: HashMap<String, RecordColumnValueJson>,
}

impl RecordResJson {
    pub fn new(data: &HashMap<String, RecordColumnValueJson>) -> Self {
        Self { data: data.clone() }
    }
}

#[derive(Serialize, Clone)]
pub enum RecordColumnValueJson {
    Boolean(Option<bool>),
    TinyInteger(Option<i8>),
    SmallInteger(Option<i16>),
    Integer(Option<i32>),
    BigInteger(Option<i64>),
    Float(Option<f32>),
    Double(Option<f64>),
    String(Option<String>),
    Byte(Option<Vec<u8>>),
    Uuid(Option<Uuid>),
    Date(Option<NaiveDate>),
    Time(Option<NaiveTime>),
    DateTime(Option<DateTime<FixedOffset>>),
    Timestamp(Option<DateTime<FixedOffset>>),
    Json(Option<Vec<u8>>),
}

impl RecordColumnValueJson {
    pub fn from_dao(value: &ColumnValue) -> Self {
        match value {
            ColumnValue::Boolean(data) => Self::Boolean(*data),
            ColumnValue::TinyInteger(data) => Self::TinyInteger(*data),
            ColumnValue::SmallInteger(data) => Self::SmallInteger(*data),
            ColumnValue::Integer(data) => Self::Integer(*data),
            ColumnValue::BigInteger(data) => Self::BigInteger(*data),
            ColumnValue::Float(data) => Self::Float(*data),
            ColumnValue::Double(data) => Self::Double(*data),
            ColumnValue::String(data) => Self::String(data.to_owned()),
            ColumnValue::Byte(data) => Self::Byte(data.to_owned()),
            ColumnValue::Uuid(data) => Self::Uuid(*data),
            ColumnValue::Date(data) => Self::Date(*data),
            ColumnValue::Time(data) => Self::Time(*data),
            ColumnValue::DateTime(data) => Self::DateTime(*data),
            ColumnValue::Timestamp(data) => Self::Timestamp(*data),
            ColumnValue::Json(data) => Self::Json(data.to_owned()),
        }
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
