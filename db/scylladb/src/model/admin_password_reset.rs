use scylla::frame::value::Timestamp;
use uuid::Uuid;

pub struct AdminPasswordResetScyllaModel {
    id: Uuid,
    created_at: Timestamp,
    updated_at: Timestamp,
    email: String,
    code: String,
}

impl AdminPasswordResetScyllaModel {
    pub fn new(
        id: Uuid,
        created_at: Timestamp,
        updated_at: Timestamp,
        email: String,
        code: String,
    ) -> Self {
        Self {
            id,
            created_at,
            updated_at,
            email,
            code,
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn created_at(&self) -> &Timestamp {
        &self.created_at
    }

    pub fn updated_at(&self) -> &Timestamp {
        &self.updated_at
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn code(&self) -> &str {
        &self.code
    }
}
