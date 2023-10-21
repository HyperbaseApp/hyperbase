use scylla::frame::value::Timestamp;
use uuid::Uuid;

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
        id: Uuid,
        created_at: Timestamp,
        updated_at: Timestamp,
        email: String,
        password_hash: String,
        code: String,
    ) -> Self {
        Self {
            id,
            created_at,
            updated_at,
            email,
            password_hash,
            code,
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
