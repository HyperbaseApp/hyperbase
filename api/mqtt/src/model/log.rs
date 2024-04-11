use chrono::{DateTime, Utc};
use hb_dao::log::LogDao;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct LogJson {
    id: Uuid,
    created_at: DateTime<Utc>,
    kind: String,
    message: String,
}

impl LogJson {
    pub fn from_dao(log: &LogDao) -> Self {
        Self {
            id: *log.id(),
            created_at: *log.created_at(),
            kind: log.kind().to_str().to_owned(),
            message: log.message().to_owned(),
        }
    }
}
