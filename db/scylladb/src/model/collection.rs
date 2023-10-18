use uuid::Uuid;

use super::{base::BaseScyllaModel, schema_field::SchemaFieldModel};

pub struct CollectionScyllaModel {
    base_model: BaseScyllaModel,
    project_id: Uuid,
    name: String,
    schema_fields: Vec<SchemaFieldModel>,
    indexes: Vec<String>,
}

impl CollectionScyllaModel {
    fn base_model(&self) -> &BaseScyllaModel {
        &self.base_model
    }

    fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn schema_fields(&self) -> &Vec<SchemaFieldModel> {
        &self.schema_fields
    }

    fn indexes(&self) -> &Vec<String> {
        &self.indexes
    }
}
