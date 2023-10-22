use chrono::{DateTime, Utc};
use hb_db_scylladb::model::project::ProjectScyllaModel;
use scylla::frame::value::Timestamp;
use uuid::Uuid;

use crate::util::conversion::datetime_to_duration_since_epoch;

pub struct ProjectDao {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    admin_id: Uuid,
    name: String,
}

impl ProjectDao {
    pub fn to_scylladb_model(&self) -> ProjectScyllaModel {
        ProjectScyllaModel::new(
            self.id,
            Timestamp(datetime_to_duration_since_epoch(self.created_at)),
            Timestamp(datetime_to_duration_since_epoch(self.updated_at)),
            self.admin_id,
            self.name.clone(),
        )
    }
}
