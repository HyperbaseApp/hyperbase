use scylla::{frame::value::Timestamp, FromRow, ValueList};
use uuid::Uuid;

#[derive(ValueList, FromRow)]
pub struct RegistrationScyllaModel {
    id: Uuid,
    created_at: Timestamp,
    updated_at: Timestamp,
    email: String,
    password_hash: String,
    code: String,
}

impl RegistrationScyllaModel {
    pub fn new(
        id: &Uuid,
        created_at: &Timestamp,
        updated_at: &Timestamp,
        email: &str,
        password_hash: &str,
        code: &str,
    ) -> Self {
        Self {
            id: *id,
            created_at: *created_at,
            updated_at: *updated_at,
            email: email.to_owned(),
            password_hash: password_hash.to_owned(),
            code: code.to_owned(),
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

    pub fn code(&self) -> &str {
        &self.code
    }
}
