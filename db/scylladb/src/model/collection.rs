use hb_db::model::{
    BaseModel as DbBaseModel, CollectionModel as DbCollectionModel,
    SchemaFieldModel as DbSchemaFieldModel,
};
use uuid::Uuid;

use super::base::BaseModel;

pub struct CollectionModel {
    base_model: BaseModel,
    project_id: Uuid,
    name: String,
    schema_fields: Vec<Box<dyn DbSchemaFieldModel>>,
    indexes: Vec<String>,
}

impl DbCollectionModel for CollectionModel {
    fn base_model(&self) -> &dyn DbBaseModel {
        &self.base_model
    }

    fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn schema_fields(&self) -> &Vec<Box<dyn DbSchemaFieldModel>> {
        &self.schema_fields
    }

    fn indexes(&self) -> &Vec<String> {
        &self.indexes
    }
}
