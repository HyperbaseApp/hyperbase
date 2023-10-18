use scylla::{frame::value::Timestamp, FromRow, ValueList};
use uuid::Uuid;

#[derive(ValueList, FromRow)]
pub struct AdminScyllaModel {
    id: Uuid,
    created_at: Timestamp,
    updated_at: Timestamp,
    email: String,
    password_hash: String,
}

impl AdminScyllaModel {
    pub fn new(
        id: Uuid,
        created_at: Timestamp,
        updated_at: Timestamp,
        email: String,
        password_hash: String,
    ) -> Self {
        Self {
            id,
            created_at,
            updated_at,
            email,
            password_hash,
        }
    }
}

impl AdminScyllaModel {
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

    pub fn password_hash(&self) -> &str {
        &self.password_hash
    }
}
