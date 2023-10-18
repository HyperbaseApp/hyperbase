use chrono::{DateTime, Utc};
use hb_db_scylladb::model::project::ProjectScyllaModel;
use scylla::frame::value::Timestamp;
use uuid::Uuid;

use crate::util::conversion::datetime_to_duration_since_epoch;

pub struct ProjectModel {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub admin_id: Uuid,
    pub name: String,
}

impl ProjectModel {
    pub fn to_scylla_model(&self) -> ProjectScyllaModel {
        ProjectScyllaModel::new(
            self.id,
            Timestamp(datetime_to_duration_since_epoch(self.created_at)),
            Timestamp(datetime_to_duration_since_epoch(self.updated_at)),
            self.admin_id,
            self.name.clone(),
        )
    }
}
