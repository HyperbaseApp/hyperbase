use hb_db::model::{AdminModel as DbAdminModel, BaseModel as DbBaseModel};

use super::base::BaseModel;

pub struct AdminModel {
    base_model: BaseModel,
    email: String,
    password_hash: String,
}

impl DbAdminModel for AdminModel {
    fn base_model(&self) -> &dyn DbBaseModel {
        &self.base_model
    }

    fn email(&self) -> &str {
        &self.email
    }

    fn password_hash(&self) -> &str {
        &self.password_hash
    }
}
