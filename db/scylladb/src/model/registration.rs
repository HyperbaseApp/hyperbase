use scylla::{frame::value::Timestamp, FromRow, ValueList};
use uuid::Uuid;

use super::admin::AdminScyllaRole;

#[derive(ValueList, FromRow)]
pub struct RegistrationScyllaModel {
    id: Uuid,
    created_at: Timestamp,
    updated_at: Timestamp,
    email: String,
    password_hash: String,
    role: AdminScyllaRole,
    code: String,
}

impl RegistrationScyllaModel {
    pub fn new(
        id: &Uuid,
        created_at: &Timestamp,
        updated_at: &Timestamp,
        email: &str,
        password_hash: &str,
        role: &AdminScyllaRole,
        code: &str,
    ) -> Self {
        Self {
            id: *id,
            created_at: *created_at,
            updated_at: *updated_at,
            email: email.to_string(),
            password_hash: password_hash.to_string(),
            role: *role,
            code: code.to_string(),
        }
    }

    pub fn set_code(&mut self, code: String) {
        self.code = code;
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

    pub fn password_hash(&self) -> &str {
        &self.password_hash
    }

    pub fn role(&self) -> &AdminScyllaRole {
        &self.role
    }

    pub fn code(&self) -> &str {
        &self.code
    }
}
