use validator::Validate;

use super::base::BaseScyllaModel;

#[derive(Validate)]
pub struct AdminScyllaModel {
    base_model: BaseScyllaModel,
    #[validate(email)]
    email: String,
    password_hash: String,
}

impl AdminScyllaModel {
    fn base_model(&self) -> &BaseScyllaModel {
        &self.base_model
    }

    fn email(&self) -> &str {
        &self.email
    }

    fn password_hash(&self) -> &str {
        &self.password_hash
    }
}
