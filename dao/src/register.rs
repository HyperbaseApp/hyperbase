use chrono::{DateTime, Utc};
use hb_db_scylladb::model::registration::RegistrationScyllaModel;
use scylla::frame::value::Timestamp;
use uuid::Uuid;

use crate::util::conversion::datetime_to_duration_since_epoch;

pub struct RegistrationModel {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    email: String,
    password_hash: String,
    code: String,
}

impl RegistrationModel {
    pub fn to_scylla_model(&self) -> RegistrationScyllaModel {
        RegistrationScyllaModel::new(
            self.id,
            Timestamp(datetime_to_duration_since_epoch(self.created_at)),
            Timestamp(datetime_to_duration_since_epoch(self.updated_at)),
            self.email.clone(),
            self.password_hash.clone(),
            self.code.clone(),
        )
    }
}
