use scylla::{frame::value::CqlTimestamp, FromRow, SerializeRow};
use uuid::Uuid;

#[derive(FromRow, SerializeRow)]
pub struct LogModel {
    id: Uuid,
    created_at: CqlTimestamp,
    admin_id: Uuid,
    project_id: Uuid,
    kind: String,
    message: String,
}

impl LogModel {
    pub fn new(
        id: &Uuid,
        created_at: &CqlTimestamp,
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

    pub fn created_at(&self) -> &CqlTimestamp {
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
