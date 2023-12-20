use ahash::HashMap;
use sqlx::{
    types::{
        chrono::{DateTime, Utc},
        Json,
    },
    FromRow,
};
use uuid::Uuid;

#[derive(FromRow)]
pub struct TokenModel {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    admin_id: Uuid,
    token: String,
    rules: Json<HashMap<Uuid, i8>>, // 0: no access, 1: read only, 2: read and write
    expired_at: Json<Option<DateTime<Utc>>>,
}

impl TokenModel {
    pub fn new(
        id: &Uuid,
        created_at: &DateTime<Utc>,
        updated_at: &DateTime<Utc>,
        admin_id: &Uuid,
        token: &str,
        rules: &HashMap<Uuid, i8>,
        expired_at: &Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id: *id,
            created_at: *created_at,
            updated_at: *updated_at,
            admin_id: *admin_id,
            token: token.to_owned(),
            rules: Json(rules.clone()),
            expired_at: Json(*expired_at),
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn updated_at(&self) -> &DateTime<Utc> {
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

    pub fn expired_at(&self) -> &Option<DateTime<Utc>> {
        &self.expired_at
    }
}
