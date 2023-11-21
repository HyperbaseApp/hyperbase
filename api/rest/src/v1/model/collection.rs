use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct InsertOneCollectionReqPath {
    project_id: Uuid,
}

impl InsertOneCollectionReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }
}

#[derive(Deserialize)]
pub struct InsertOneCollectionReqJson {
    name: String,
    #[serde(rename = "schemaFields")]
    schema_fields: HashMap<String, SchemaFieldModelJson>,
    indexes: Vec<String>,
}

impl InsertOneCollectionReqJson {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn schema_fields(&self) -> &HashMap<String, SchemaFieldModelJson> {
        &self.schema_fields
    }

    pub fn indexes(&self) -> &Vec<String> {
        &self.indexes
    }
}

#[derive(Deserialize)]
pub struct FindOneCollectionReqPath {
    project_id: Uuid,
    collection_id: Uuid,
}

impl FindOneCollectionReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn collection_id(&self) -> &Uuid {
        &self.collection_id
    }
}

#[derive(Deserialize)]
pub struct UpdateOneCollectionReqPath {
    project_id: Uuid,
    collection_id: Uuid,
}

impl UpdateOneCollectionReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn collection_id(&self) -> &Uuid {
        &self.collection_id
    }
}

#[derive(Deserialize)]
pub struct UpdateOneCollectionReqJson {
    name: Option<String>,
    #[serde(rename = "schemaFields")]
    schema_fields: Option<HashMap<String, SchemaFieldModelJson>>,
    indexes: Option<Vec<String>>,
}

impl UpdateOneCollectionReqJson {
    pub fn name(&self) -> &Option<String> {
        &self.name
    }

    pub fn schema_fields(&self) -> &Option<HashMap<String, SchemaFieldModelJson>> {
        &self.schema_fields
    }

    pub fn indexes(&self) -> &Option<Vec<String>> {
        &self.indexes
    }
}

impl UpdateOneCollectionReqJson {
    pub fn is_all_none(&self) -> bool {
        self.name.is_none() && self.schema_fields.is_none() && self.indexes.is_none()
    }
}

#[derive(Deserialize)]
pub struct DeleteOneCollectionReqPath {
    project_id: Uuid,
    collection_id: Uuid,
}

impl DeleteOneCollectionReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn collection_id(&self) -> &Uuid {
        &self.collection_id
    }
}

#[derive(Serialize)]
pub struct CollectionResJson {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    project_id: Uuid,
    name: String,
    schema_fields: HashMap<String, SchemaFieldModelJson>,
    indexes: Vec<String>,
}

impl CollectionResJson {
    pub fn new(
        id: &Uuid,
        created_at: &DateTime<Utc>,
        updated_at: &DateTime<Utc>,
        project_id: &Uuid,
        name: &str,
        schema_fields: &HashMap<String, SchemaFieldModelJson>,
        indexes: &Vec<String>,
    ) -> Self {
        Self {
            id: *id,
            created_at: *created_at,
            updated_at: *updated_at,
            project_id: *project_id,
            name: name.to_string(),
            schema_fields: schema_fields.to_owned(),
            indexes: indexes.to_vec(),
        }
    }
}

#[derive(Serialize)]
pub struct DeleteCollectionResJson {
    id: Uuid,
}

impl DeleteCollectionResJson {
    pub fn new(id: &Uuid) -> Self {
        Self { id: *id }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SchemaFieldModelJson {
    kind: String,
    required: bool,
}

impl SchemaFieldModelJson {
    pub fn new(kind: &str, required: &bool) -> Self {
        Self {
            kind: kind.to_string(),
            required: *required,
        }
    }

    pub fn kind(&self) -> &str {
        &self.kind
    }

    pub fn required(&self) -> &bool {
        &self.required
    }
}
