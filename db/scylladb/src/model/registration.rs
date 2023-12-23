use scylla::{frame::value::CqlTimestamp, FromRow, SerializeRow};
use uuid::Uuid;

#[derive(FromRow, SerializeRow)]
pub struct RegistrationModel {
    id: Uuid,
    created_at: CqlTimestamp,
    updated_at: CqlTimestamp,
    email: String,
    password_hash: String,
    code: String,
}

impl RegistrationModel {
    pub fn new(
        id: &Uuid,
        created_at: &CqlTimestamp,
        updated_at: &CqlTimestamp,
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

    pub fn set_code(&mut self, code: &str) {
        self.code = code.to_owned();
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn created_at(&self) -> &CqlTimestamp {
        &self.created_at
    }

    pub fn updated_at(&self) -> &CqlTimestamp {
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
