use ahash::{HashMap, HashSet};
use scylla::{frame::value::Timestamp, FromRow, FromUserType, IntoUserType, ValueList};
use uuid::Uuid;

use super::system::SchemaFieldScyllaKind;

#[derive(ValueList, FromRow)]
pub struct CollectionScyllaModel {
    id: Uuid,
    created_at: Timestamp,
    updated_at: Timestamp,
    project_id: Uuid,
    name: String,
    schema_fields: HashMap<String, SchemaFieldPropsScyllaModel>,
    indexes: Option<HashSet<String>>,
}

impl CollectionScyllaModel {
    pub fn new(
        id: &Uuid,
        created_at: &Timestamp,
        updated_at: &Timestamp,
        project_id: &Uuid,
        name: &str,
        schema_fields: &HashMap<String, SchemaFieldPropsScyllaModel>,
        indexes: &Option<HashSet<String>>,
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

    pub fn created_at(&self) -> &Timestamp {
        &self.created_at
    }

    pub fn updated_at(&self) -> &Timestamp {
        &self.updated_at
    }

    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn schema_fields(&self) -> &HashMap<String, SchemaFieldPropsScyllaModel> {
        &self.schema_fields
    }

    pub fn indexes(&self) -> &Option<HashSet<String>> {
        &self.indexes
    }
}

#[derive(IntoUserType, FromUserType, Clone)]
pub struct SchemaFieldPropsScyllaModel {
    kind: String,
    internal_kind: SchemaFieldScyllaKind,
    required: bool,
}

impl SchemaFieldPropsScyllaModel {
    pub fn new(kind: &str, internal_kind: &SchemaFieldScyllaKind, required: &bool) -> Self {
        Self {
            kind: kind.to_owned(),
            internal_kind: *internal_kind,
            required: *required,
        }
    }

    pub fn kind(&self) -> &str {
        &self.kind
    }

    pub fn internal_kind(&self) -> &SchemaFieldScyllaKind {
        &self.internal_kind
    }

    pub fn required(&self) -> &bool {
        &self.required
    }
}
