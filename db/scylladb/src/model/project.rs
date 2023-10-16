use hb_db::model::{BaseModel as DbBaseModel, ProjectModel as DbProjectModel};
use uuid::Uuid;

use super::base::BaseModel;

pub struct ProjectModel {
    base_model: BaseModel,
    admin_id: Uuid,
    name: String,
}

impl DbProjectModel for ProjectModel {
    fn base_model(&self) -> &dyn DbBaseModel {
        &self.base_model
    }

    fn admin_id(&self) -> &uuid::Uuid {
        &self.admin_id
    }

    fn name(&self) -> &str {
        &self.name
    }
}
