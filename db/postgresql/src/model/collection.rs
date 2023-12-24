use ahash::HashMap;
use serde::{Deserialize, Serialize};
use sqlx::{
    prelude::FromRow,
    types::{
        chrono::{DateTime, Utc},
        Json,
    },
};
use uuid::Uuid;

use super::system::SchemaFieldKind;

#[derive(FromRow)]
pub struct CollectionModel {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    project_id: Uuid,
    name: String,
    schema_fields: Json<HashMap<String, SchemaFieldPropsModel>>,
    indexes: Vec<String>,
}

impl CollectionModel {
    pub fn new(
        id: &Uuid,
        created_at: &DateTime<Utc>,
        updated_at: &DateTime<Utc>,
        project_id: &Uuid,
        name: &str,
        schema_fields: &Json<HashMap<String, SchemaFieldPropsModel>>,
        indexes: &Vec<String>,
    ) -> Self {
        Self {
            id: *id,
            created_at: *created_at,
            updated_at: *updated_at,
            project_id: *project_id,
            name: name.to_owned(),
            schema_fields: schema_fields.clone(),
            indexes: indexes.clone(),
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }

    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn schema_fields(&self) -> &Json<HashMap<String, SchemaFieldPropsModel>> {
        &self.schema_fields
    }

    pub fn indexes(&self) -> &Vec<String> {
        &self.indexes
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SchemaFieldPropsModel {
    kind: String,
    internal_kind: SchemaFieldKind,
    required: bool,
}

impl SchemaFieldPropsModel {
    pub fn new(kind: &str, internal_kind: &SchemaFieldKind, required: &bool) -> Self {
        Self {
            kind: kind.to_owned(),
            internal_kind: *internal_kind,
            required: *required,
        }
    }

    pub fn kind(&self) -> &str {
        &self.kind
    }

    pub fn internal_kind(&self) -> &SchemaFieldKind {
        &self.internal_kind
    }

    pub fn required(&self) -> &bool {
        &self.required
    }
}
