use serde::Deserialize;
use uuid::Uuid;

use super::schema_field::SchemaFieldModel;

#[derive(Deserialize)]
pub struct InsertOneCollectionPath {
    admin_id: Uuid,
    project_id: Uuid,
}

impl InsertOneCollectionPath {
    pub fn admin_id(&self) -> &Uuid {
        &self.admin_id
    }

    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }
}

#[derive(Deserialize)]
pub struct InsertOneCollectionJson {
    name: String,
    #[serde(rename = "schemaFields")]
    schema_fields: Vec<SchemaFieldModel>,
    indexes: Option<Vec<String>>,
}

impl InsertOneCollectionJson {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn schema_fields(&self) -> &Vec<SchemaFieldModel> {
        &self.schema_fields
    }

    pub fn indexes(&self) -> &Option<Vec<String>> {
        &self.indexes
    }
}

#[derive(Deserialize)]
pub struct FindOneCollectionPath {
    admin_id: Uuid,
    project_id: Uuid,
    collection_id: Uuid,
}

impl FindOneCollectionPath {
    pub fn admin_id(&self) -> &Uuid {
        &self.admin_id
    }

    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn collection_id(&self) -> &Uuid {
        &self.collection_id
    }
}

#[derive(Deserialize)]
pub struct UpdateOneCollectionPath {
    admin_id: Uuid,
    project_id: Uuid,
    collection_id: Uuid,
}

impl UpdateOneCollectionPath {
    pub fn admin_id(&self) -> &Uuid {
        &self.admin_id
    }

    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn collection_id(&self) -> &Uuid {
        &self.collection_id
    }
}

#[derive(Deserialize)]
pub struct UpdateOneCollectionJson {
    name: Option<String>,
    #[serde(rename = "schemaFields")]
    schema_fields: Option<Vec<SchemaFieldModel>>,
    indexes: Option<Vec<String>>,
}

impl UpdateOneCollectionJson {
    pub fn name(&self) -> &Option<String> {
        &self.name
    }

    pub fn schema_fields(&self) -> &Option<Vec<SchemaFieldModel>> {
        &self.schema_fields
    }

    pub fn indexes(&self) -> &Option<Vec<String>> {
        &self.indexes
    }
}

#[derive(Deserialize)]
pub struct DeleteOneCollectionPath {
    admin_id: Uuid,
    project_id: Uuid,
    collection_id: Uuid,
}

impl DeleteOneCollectionPath {
    pub fn admin_id(&self) -> &Uuid {
        &self.admin_id
    }

    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn collection_id(&self) -> &Uuid {
        &self.collection_id
    }
}
