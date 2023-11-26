use ahash::HashMap;
use scylla::{frame::value::Timestamp, FromRow, ValueList};
use uuid::Uuid;

#[derive(ValueList, FromRow)]
pub struct TokenScyllaModel {
    id: Uuid,
    created_at: Timestamp,
    updated_at: Timestamp,
    admin_id: Uuid,
    token: String,
    rules: HashMap<Uuid, i8>, // 0: no access, 1: read only, 2: read and write
    expired_at: Option<Timestamp>,
}

impl TokenScyllaModel {
    pub fn new(
        id: &Uuid,
        created_at: &Timestamp,
        updated_at: &Timestamp,
        admin_id: &Uuid,
        token: &str,
        rules: &HashMap<Uuid, i8>,
        expired_at: &Option<Timestamp>,
    ) -> Self {
        Self {
            id: *id,
            created_at: *created_at,
            updated_at: *updated_at,
            admin_id: *admin_id,
            token: token.to_owned(),
            rules: rules.clone(),
            expired_at: *expired_at,
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

    pub fn admin_id(&self) -> &Uuid {
        &self.admin_id
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn rules(&self) -> &HashMap<Uuid, i8> {
        &self.rules
    }

    pub fn expired_at(&self) -> &Option<Timestamp> {
        &self.expired_at
    }
}
