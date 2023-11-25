use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct InsertOneTokenReqJson {
    expired_at: DateTime<Utc>,
}

impl InsertOneTokenReqJson {
    pub fn expired_at(&self) -> &DateTime<Utc> {
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
    expired_at: Option<DateTime<Utc>>,
}

impl UpdateOneTokenReqJson {
    pub fn expired_at(&self) -> &Option<DateTime<Utc>> {
        &self.expired_at
    }

    pub fn is_all_none(&self) -> bool {
        self.expired_at.is_none()
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
    expired_at: DateTime<Utc>,
}

impl TokenResJson {
    pub fn new(
        id: &Uuid,
        created_at: &DateTime<Utc>,
        updated_at: &DateTime<Utc>,
        token: &str,
        expired_at: &DateTime<Utc>,
    ) -> Self {
        Self {
            id: *id,
            created_at: *created_at,
            updated_at: *updated_at,
            token: token.to_owned(),
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
