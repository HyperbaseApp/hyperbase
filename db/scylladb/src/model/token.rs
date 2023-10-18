use scylla::frame::value::Timestamp;
use uuid::Uuid;

use super::base::BaseScyllaModel;

pub struct TokenScyllaModel {
    base_model: BaseScyllaModel,
    admin_id: Uuid,
    token: String,
    expired_at: Timestamp,
}

impl TokenScyllaModel {
    fn base_model(&self) -> &BaseScyllaModel {
        &self.base_model
    }

    fn admin_id(&self) -> &Uuid {
        &self.admin_id
    }

    fn token(&self) -> &str {
        &self.token
    }

    fn expired_at(&self) -> &Timestamp {
        &self.expired_at
    }
}
