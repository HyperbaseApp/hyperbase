use super::BaseModel;

pub trait ProjectModel {
    fn table_name(&self) -> &str {
        "_projects"
    }

    fn base_model(&self) -> &dyn BaseModel;
    fn admin_id(&self) -> &uuid::Uuid;
    fn name(&self) -> &str;
}
