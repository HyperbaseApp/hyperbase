use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct FindManyLogReqPath {
    project_id: Uuid,
}

impl FindManyLogReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }
}

#[derive(Deserialize)]
pub struct FindManyLogReqQuery {
    before_id: Option<Uuid>,
    limit: Option<i32>,
}

impl FindManyLogReqQuery {
    pub fn before_id(&self) -> &Option<Uuid> {
        &self.before_id
    }

    pub fn limit(&self) -> &Option<i32> {
        &self.limit
    }
}

#[derive(Deserialize)]
pub struct SubscribeLogReqPath {
    project_id: Uuid,
}

impl SubscribeLogReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }
}

#[derive(Deserialize)]
pub struct SubscribeLogReqQuery {
    token: String,
}

impl SubscribeLogReqQuery {
    pub fn token(&self) -> &str {
        &self.token
    }
}

#[derive(Serialize)]
pub struct LogResJson {
    id: Uuid,
    created_at: DateTime<Utc>,
    kind: String,
    message: String,
}

impl LogResJson {
    pub fn new(id: &Uuid, created_at: &DateTime<Utc>, kind: &str, message: &str) -> Self {
        Self {
            id: *id,
            created_at: *created_at,
            kind: kind.to_owned(),
            message: message.to_owned(),
        }
    }
}
