use ahash::HashMap;
use scylla::{frame::value::CqlTimestamp, FromRow, FromUserType, SerializeCql, SerializeRow};
use uuid::Uuid;

use super::value::ColumnKind;

#[derive(FromRow, SerializeRow)]
pub struct CollectionModel {
    id: Uuid,
    created_at: CqlTimestamp,
    updated_at: CqlTimestamp,
    project_id: Uuid,
    name: String,
    schema_fields: HashMap<String, SchemaFieldPropsModel>,
    opt_auth_column_id: bool,
}

impl CollectionModel {
    pub fn new(
        id: &Uuid,
        created_at: &CqlTimestamp,
        updated_at: &CqlTimestamp,
        project_id: &Uuid,
        name: &str,
        schema_fields: &HashMap<String, SchemaFieldPropsModel>,
        opt_auth_column_id: &bool,
    ) -> Self {
        Self {
            id: *id,
            created_at: *created_at,
            updated_at: *updated_at,
            project_id: *project_id,
            name: name.to_owned(),
            schema_fields: schema_fields.clone(),
            opt_auth_column_id: *opt_auth_column_id,
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn created_at(&self) -> &CqlTimestamp {
        &self.created_at
    }

    pub fn updated_at(&self) -> &CqlTimestamp {
        &self.updated_at
    }

    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn schema_fields(&self) -> &HashMap<String, SchemaFieldPropsModel> {
        &self.schema_fields
    }

    pub fn opt_auth_column_id(&self) -> &bool {
        &self.opt_auth_column_id
    }
}

#[derive(FromUserType, SerializeCql, Clone)]
pub struct SchemaFieldPropsModel {
    kind: String,
    internal_kind: ColumnKind,
    required: bool,
    unqiue: bool,
    indexed: bool,
    auth_column: bool,
}

impl SchemaFieldPropsModel {
    pub fn new(
        kind: &str,
        internal_kind: &ColumnKind,
        required: &bool,
        unique: &bool,
        indexed: &bool,
        auth_column: &bool,
    ) -> Self {
        Self {
            kind: kind.to_owned(),
            internal_kind: *internal_kind,
            required: *required,
            unqiue: *unique,
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

    pub fn unique(&self) -> &bool {
        &self.unqiue
    }

    pub fn indexed(&self) -> &bool {
        &self.indexed
    }

    pub fn auth_column(&self) -> &bool {
        &self.auth_column
    }
}
