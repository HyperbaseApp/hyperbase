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

use super::value::ColumnKind;

#[derive(FromRow)]
pub struct CollectionModel {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    project_id: Uuid,
    name: String,
    schema_fields: Json<HashMap<String, SchemaFieldPropsModel>>,
}

impl CollectionModel {
    pub fn new(
        id: &Uuid,
        created_at: &DateTime<Utc>,
        updated_at: &DateTime<Utc>,
        project_id: &Uuid,
        name: &str,
        schema_fields: &Json<HashMap<String, SchemaFieldPropsModel>>,
    ) -> Self {
        Self {
            id: *id,
            created_at: *created_at,
            updated_at: *updated_at,
            project_id: *project_id,
            name: name.to_owned(),
            schema_fields: schema_fields.clone(),
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
}

#[derive(Deserialize, Serialize, Clone)]
pub struct SchemaFieldPropsModel {
    kind: String,
    internal_kind: ColumnKind,
    required: bool,
    indexed: bool,
    auth_column: bool,
}

impl SchemaFieldPropsModel {
    pub fn new(
        kind: &str,
        internal_kind: &ColumnKind,
        required: &bool,
        indexed: &bool,
        auth_column: &bool,
    ) -> Self {
        Self {
            kind: kind.to_owned(),
            internal_kind: *internal_kind,
            required: *required,
            indexed: *indexed,
            auth_column: *auth_column,
        }
    }

    pub fn kind(&self) -> &str {
        &self.kind
    }

    pub fn internal_kind(&self) -> &ColumnKind {
        &self.internal_kind
    }

    pub fn required(&self) -> &bool {
        &self.required
    }

    pub fn indexed(&self) -> &bool {
        &self.indexed
    }

    pub fn auth_column(&self) -> &bool {
        &self.auth_column
    }
}
