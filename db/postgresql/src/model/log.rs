use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(FromRow)]
pub struct LogModel {
    id: Uuid,
    created_at: DateTime<Utc>,
    admin_id: Uuid,
    project_id: Uuid,
    kind: String,
    message: String,
}

impl LogModel {
    pub fn new(
        id: &Uuid,
        created_at: &DateTime<Utc>,
        admin_id: &Uuid,
        project_id: &Uuid,
        kind: &str,
        message: &str,
    ) -> Self {
        Self {
            id: *id,
            created_at: *created_at,
            admin_id: *admin_id,
            project_id: *project_id,
            kind: kind.to_owned(),
            message: message.to_owned(),
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn admin_id(&self) -> &Uuid {
        &self.admin_id
    }

    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn kind(&self) -> &str {
        &self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}
