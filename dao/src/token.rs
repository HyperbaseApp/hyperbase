use chrono::{DateTime, Utc};
use hb_db_scylladb::model::token::TokenScyllaModel;
use scylla::frame::value::Timestamp;
use uuid::Uuid;

use crate::util::conversion::datetime_to_duration_since_epoch;

pub struct TokenModel {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    admin_id: Uuid,
    token: String,
    expired_at: DateTime<Utc>,
}

impl TokenModel {
    pub fn to_scylla_model(&self) -> TokenScyllaModel {
        TokenScyllaModel::new(
            self.id,
            Timestamp(datetime_to_duration_since_epoch(self.created_at)),
            Timestamp(datetime_to_duration_since_epoch(self.updated_at)),
            self.admin_id,
            self.token.clone(),
            Timestamp(datetime_to_duration_since_epoch(self.expired_at)),
        )
    }
}
