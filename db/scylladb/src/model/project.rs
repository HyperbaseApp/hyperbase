use scylla::{frame::value::Timestamp, FromRow, ValueList};
use uuid::Uuid;

#[derive(ValueList, FromRow)]
pub struct ProjectScyllaModel {
    id: Uuid,
    created_at: Timestamp,
    updated_at: Timestamp,
    admin_id: Uuid,
    name: String,
}

impl ProjectScyllaModel {
    pub fn new(
        id: Uuid,
        created_at: Timestamp,
        updated_at: Timestamp,
        admin_id: Uuid,
        name: String,
    ) -> Self {
        Self {
            id,
            created_at,
            updated_at,
            admin_id,
            name,
        }
    }
}

impl ProjectScyllaModel {
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn created_at(&self) -> &Timestamp {
        &self.created_at
    }

    pub fn updated_at(&self) -> &Timestamp {
        &self.updated_at
    }

    pub fn admin_id(&self) -> &Uuid {
        &self.admin_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
