use ahash::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct InsertOneTokenReqJson {
    rules: HashMap<Uuid, i8>,
    expired_at: Option<DateTime<Utc>>,
}

impl InsertOneTokenReqJson {
    pub fn rules(&self) -> &HashMap<Uuid, i8> {
        &self.rules
    }

    pub fn expired_at(&self) -> &Option<DateTime<Utc>> {
        &self.expired_at
    }
}

#[derive(Deserialize)]
pub struct FindOneTokenReqPath {
    token_id: Uuid,
}

impl FindOneTokenReqPath {
    pub fn token_id(&self) -> &Uuid {
        &self.token_id
    }
}

#[derive(Deserialize)]
pub struct UpdateOneTokenReqPath {
    token_id: Uuid,
}

impl UpdateOneTokenReqPath {
    pub fn token_id(&self) -> &Uuid {
        &self.token_id
    }
}

#[derive(Deserialize)]
pub struct UpdateOneTokenReqJson {
    rules: Option<HashMap<Uuid, i8>>,
    expired_at: Option<DateTime<Utc>>,
}

impl UpdateOneTokenReqJson {
    pub fn rules(&self) -> &Option<HashMap<Uuid, i8>> {
        &self.rules
    }

    pub fn expired_at(&self) -> &Option<DateTime<Utc>> {
        &self.expired_at
    }
}

#[derive(Deserialize)]
pub struct DeleteOneTokenReqPath {
    token_id: Uuid,
}

impl DeleteOneTokenReqPath {
    pub fn token_id(&self) -> &Uuid {
        &self.token_id
    }
}

#[derive(Serialize)]
pub struct TokenResJson {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    token: String,
    rules: HashMap<Uuid, i8>,
    expired_at: Option<DateTime<Utc>>,
}

impl TokenResJson {
    pub fn new(
        id: &Uuid,
        created_at: &DateTime<Utc>,
        updated_at: &DateTime<Utc>,
        token: &str,
        rules: &HashMap<Uuid, i8>,
        expired_at: &Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id: *id,
            created_at: *created_at,
            updated_at: *updated_at,
            token: token.to_owned(),
            rules: rules.clone(),
            expired_at: *expired_at,
        }
    }
}

#[derive(Serialize)]
pub struct DeleteTokenResJson {
    id: Uuid,
}

impl DeleteTokenResJson {
    pub fn new(id: &Uuid) -> Self {
        Self { id: *id }
    }
}
