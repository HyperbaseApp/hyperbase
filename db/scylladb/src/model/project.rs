use uuid::Uuid;

use super::base::BaseScyllaModel;

pub struct ProjectScyllaModel {
    base_model: BaseScyllaModel,
    admin_id: Uuid,
    name: String,
}

impl ProjectScyllaModel {
    fn base_model(&self) -> &BaseScyllaModel {
        &self.base_model
    }

    fn admin_id(&self) -> &Uuid {
        &self.admin_id
    }

    fn name(&self) -> &str {
        &self.name
    }
}
