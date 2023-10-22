use chrono::{DateTime, Utc};
use hb_db_scylladb::model::admin_password_reset::AdminPasswordResetScyllaModel;
use scylla::frame::value::Timestamp;
use uuid::Uuid;

use crate::util::conversion::datetime_to_duration_since_epoch;

pub struct AdminPasswordResetDao {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    email: String,
    code: String,
}

impl AdminPasswordResetDao {
    pub fn to_scylladb_model(&self) -> AdminPasswordResetScyllaModel {
        AdminPasswordResetScyllaModel::new(
            self.id,
            Timestamp(datetime_to_duration_since_epoch(self.created_at)),
            Timestamp(datetime_to_duration_since_epoch(self.updated_at)),
            self.email.clone(),
            self.code.clone(),
        )
    }
}
