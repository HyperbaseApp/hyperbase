use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub enum HeaderMessage {
    Request {
        from_time: DateTime<Utc>,
        last_change_id: Uuid,
    },
    Response {
        change_ids: Vec<Uuid>,
    },
}
