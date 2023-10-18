use chrono::{DateTime, Utc};
use hb_db_scylladb::model::collection::{
    CollectionScyllaModel, SchemaScyllaFieldKind, SchemaScyllaFieldModel,
};
use scylla::frame::value::Timestamp;
use uuid::Uuid;

use crate::util::conversion::datetime_to_duration_since_epoch;

pub struct CollectionModel {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub project_id: Uuid,
    pub name: String,
    pub schema_fields: Vec<SchemaFieldModel>,
    pub indexes: Vec<String>,
}

impl CollectionModel {
    pub fn to_scylla_model(&self) -> CollectionScyllaModel {
        CollectionScyllaModel::new(
            self.id,
            Timestamp(datetime_to_duration_since_epoch(self.created_at)),
            Timestamp(datetime_to_duration_since_epoch(self.updated_at)),
            self.project_id,
            self.name.clone(),
            self.schema_fields
                .clone()
                .into_iter()
                .map(|schema_field| schema_field.to_scylla_model())
                .collect(),
            self.indexes.clone(),
        )
    }
}

#[derive(Clone)]
pub struct SchemaFieldModel {
    pub name: String,
    pub kind: SchemaFieldKind,
    pub required: bool,
}

impl SchemaFieldModel {
    pub fn to_scylla_model(self) -> SchemaScyllaFieldModel {
        SchemaScyllaFieldModel::new(self.name, self.kind.to_scylla_model(), self.required)
    }
}

#[derive(Clone)]
pub enum SchemaFieldKind {
    Integer,
    Float,
    String,
}

impl SchemaFieldKind {
    pub fn to_scylla_model(&self) -> SchemaScyllaFieldKind {
        match self {
            Self::Integer => SchemaScyllaFieldKind::Int,
            Self::Float => SchemaScyllaFieldKind::Double,
            Self::String => SchemaScyllaFieldKind::Text,
        }
    }
}
