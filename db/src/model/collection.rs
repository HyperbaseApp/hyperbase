use super::{base::BaseModel, SchemaFieldModel};

pub trait CollectionModel {
    fn table_name(&self) -> &str {
        "_collections"
    }

    fn base_model(&self) -> &dyn BaseModel;
    fn project_id(&self) -> &uuid::Uuid;
    fn name(&self) -> &str;
    fn schema_fields(&self) -> &Vec<Box<dyn SchemaFieldModel>>;
    fn indexes(&self) -> &Vec<String>;
}
