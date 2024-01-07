use ahash::HashMap;
use anyhow::Result;
use chrono::{DateTime, Utc};
use hb_dao::token::TokenRuleMethod;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct InsertOneTokenReqJson {
    rules: HashMap<Uuid, TokenRuleMethodJson>,
    expired_at: Option<DateTime<Utc>>,
}

impl InsertOneTokenReqJson {
    pub fn rules(&self) -> &HashMap<Uuid, TokenRuleMethodJson> {
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
    rules: Option<HashMap<Uuid, TokenRuleMethodJson>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    expired_at: Option<Option<DateTime<Utc>>>,
}

impl UpdateOneTokenReqJson {
    pub fn rules(&self) -> &Option<HashMap<Uuid, TokenRuleMethodJson>> {
        &self.rules
    }

    pub fn expired_at(&self) -> &Option<Option<DateTime<Utc>>> {
        &self.expired_at
    }

    pub fn is_all_none(&self) -> bool {
        self.rules.is_none() && self.expired_at.is_none()
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
    rules: HashMap<Uuid, TokenRuleMethodJson>,
    expired_at: Option<DateTime<Utc>>,
}

impl TokenResJson {
    pub fn new(
        id: &Uuid,
        created_at: &DateTime<Utc>,
        updated_at: &DateTime<Utc>,
        token: &str,
        rules: &HashMap<Uuid, TokenRuleMethodJson>,
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

#[derive(Deserialize, Serialize, Clone)]
pub struct TokenRuleMethodJson {
    find_one: Option<bool>,
    find_many: Option<bool>,
    insert: Option<bool>,
    update: Option<bool>,
    delete: Option<bool>,
}

impl TokenRuleMethodJson {
    pub fn from_dao(dao: &TokenRuleMethod) -> Result<Self> {
        Ok(Self {
            find_one: Some(*dao.find_one()),
            find_many: Some(*dao.find_many()),
            insert: Some(*dao.insert()),
            update: Some(*dao.update()),
            delete: Some(*dao.delete()),
        })
    }

    pub fn to_dao(&self) -> Result<TokenRuleMethod> {
        Ok(TokenRuleMethod::new(
            &match self.find_one {
                Some(find_one) => find_one,
                None => false,
            },
            &match self.find_many {
                Some(find_many) => find_many,
                None => false,
            },
            &match self.insert {
                Some(insert) => insert,
                None => false,
            },
            &match self.update {
                Some(update) => update,
                None => false,
            },
            &match self.delete {
                Some(delete) => delete,
                None => false,
            },
        ))
    }
}
