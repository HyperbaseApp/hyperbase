use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct InsertOneTokenReqPath {
    project_id: Uuid,
}

impl InsertOneTokenReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }
}

#[derive(Deserialize)]
pub struct InsertOneTokenReqJson {
    name: String,
    allow_anonymous: bool,
    expired_at: Option<DateTime<Utc>>,
}

impl InsertOneTokenReqJson {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn allow_anonymous(&self) -> &bool {
        &self.allow_anonymous
    }

    pub fn expired_at(&self) -> &Option<DateTime<Utc>> {
        &self.expired_at
    }
}

#[derive(Deserialize)]
pub struct FindOneTokenReqPath {
    project_id: Uuid,
    token_id: Uuid,
}

impl FindOneTokenReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn token_id(&self) -> &Uuid {
        &self.token_id
    }
}

#[derive(Deserialize)]
pub struct UpdateOneTokenReqPath {
    project_id: Uuid,
    token_id: Uuid,
}

impl UpdateOneTokenReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn token_id(&self) -> &Uuid {
        &self.token_id
    }
}

#[derive(Deserialize)]
pub struct UpdateOneTokenReqJson {
    name: Option<String>,
    allow_anonymous: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    expired_at: Option<Option<DateTime<Utc>>>,
}

impl UpdateOneTokenReqJson {
    pub fn name(&self) -> &Option<String> {
        &self.name
    }

    pub fn allow_anonymous(&self) -> &Option<bool> {
        &self.allow_anonymous
    }

    pub fn expired_at(&self) -> &Option<Option<DateTime<Utc>>> {
        &self.expired_at
    }

    pub fn is_all_none(&self) -> bool {
        self.name.is_none() && self.allow_anonymous.is_none() && self.expired_at.is_none()
    }
}

#[derive(Deserialize)]
pub struct DeleteOneTokenReqPath {
    project_id: Uuid,
    token_id: Uuid,
}

impl DeleteOneTokenReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn token_id(&self) -> &Uuid {
        &self.token_id
    }
}

#[derive(Deserialize)]
pub struct FindManyTokenReqPath {
    project_id: Uuid,
}

impl FindManyTokenReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }
}

#[derive(Serialize)]
pub struct TokenResJson {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    project_id: Uuid,
    name: String,
    token: String,
    allow_anonymous: bool,
    expired_at: Option<DateTime<Utc>>,
}

impl TokenResJson {
    pub fn new(
        id: &Uuid,
        created_at: &DateTime<Utc>,
        updated_at: &DateTime<Utc>,
        project_id: &Uuid,
        name: &str,
        token: &str,
        allow_anonymous: &bool,
        expired_at: &Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id: *id,
            created_at: *created_at,
            updated_at: *updated_at,
            project_id: *project_id,
            name: name.to_owned(),
            token: token.to_owned(),
            allow_anonymous: *allow_anonymous,
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
