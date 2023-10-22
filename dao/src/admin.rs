use chrono::{DateTime, Utc};
use hb_db_scylladb::model::admin::AdminScyllaModel;
use scylla::frame::value::Timestamp;
use uuid::Uuid;

use crate::util::conversion::datetime_to_duration_since_epoch;

pub struct AdminDao {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    email: String,
    password_hash: String,
}

impl AdminDao {
    pub fn to_scylladb_model(&self) -> AdminScyllaModel {
        AdminScyllaModel::new(
            self.id,
            Timestamp(datetime_to_duration_since_epoch(self.created_at)),
            Timestamp(datetime_to_duration_since_epoch(self.updated_at)),
            self.email.clone(),
            self.password_hash.clone(),
        )
    }
}
