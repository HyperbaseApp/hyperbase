use serde::Deserialize;
use uuid::Uuid;

use super::schema_field::SchemaFieldModel;

#[derive(Deserialize)]
pub struct InsertOneCollectionReqPath {
    admin_id: Uuid,
    project_id: Uuid,
}

impl InsertOneCollectionReqPath {
    pub fn admin_id(&self) -> &Uuid {
        &self.admin_id
    }

    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }
}

#[derive(Deserialize)]
pub struct InsertOneCollectionReqJson {
    name: String,
    #[serde(rename = "schemaFields")]
    schema_fields: Vec<SchemaFieldModel>,
    indexes: Option<Vec<String>>,
}

impl InsertOneCollectionReqJson {
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
pub struct FindOneCollectionReqPath {
    admin_id: Uuid,
    project_id: Uuid,
    collection_id: Uuid,
}

impl FindOneCollectionReqPath {
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
pub struct UpdateOneCollectionReqPath {
    admin_id: Uuid,
    project_id: Uuid,
    collection_id: Uuid,
}

impl UpdateOneCollectionReqPath {
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
pub struct UpdateOneCollectionReqJson {
    name: Option<String>,
    #[serde(rename = "schemaFields")]
    schema_fields: Option<Vec<SchemaFieldModel>>,
    indexes: Option<Vec<String>>,
}

impl UpdateOneCollectionReqJson {
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
pub struct DeleteOneCollectionReqPath {
    admin_id: Uuid,
    project_id: Uuid,
    collection_id: Uuid,
}

impl DeleteOneCollectionReqPath {
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
