use ahash::HashMap;
use scylla::{frame::value::CqlTimestamp, FromRow, SerializeRow};
use uuid::Uuid;

#[derive(FromRow, SerializeRow)]
pub struct TokenModel {
    id: Uuid,
    created_at: CqlTimestamp,
    updated_at: CqlTimestamp,
    admin_id: Uuid,
    token: String,
    rules: HashMap<Uuid, i8>, // 0: no access, 1: read only, 2: read and write
    expired_at: Option<CqlTimestamp>,
}

impl TokenModel {
    pub fn new(
        id: &Uuid,
        created_at: &CqlTimestamp,
        updated_at: &CqlTimestamp,
        admin_id: &Uuid,
        token: &str,
        rules: &HashMap<Uuid, i8>,
        expired_at: &Option<CqlTimestamp>,
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

    pub fn created_at(&self) -> &CqlTimestamp {
        &self.created_at
    }

    pub fn updated_at(&self) -> &CqlTimestamp {
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

    pub fn expired_at(&self) -> &Option<CqlTimestamp> {
        &self.expired_at
    }
}
