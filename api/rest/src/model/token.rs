use ahash::HashMap;
use anyhow::Result;
use chrono::{DateTime, Utc};
use hb_dao::token::{TokenBucketRuleMethod, TokenCollectionRuleMethod};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct InsertOneTokenReqJson {
    bucket_rules: Option<HashMap<Uuid, TokenBucketRuleMethodJson>>,
    collection_rules: Option<HashMap<Uuid, TokenCollectionRuleMethodJson>>,
    expired_at: Option<DateTime<Utc>>,
}

impl InsertOneTokenReqJson {
    pub fn bucket_rules(&self) -> &Option<HashMap<Uuid, TokenBucketRuleMethodJson>> {
        &self.bucket_rules
    }

    pub fn collection_rules(&self) -> &Option<HashMap<Uuid, TokenCollectionRuleMethodJson>> {
        &self.collection_rules
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
    collection_rules: Option<HashMap<Uuid, TokenCollectionRuleMethodJson>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    expired_at: Option<Option<DateTime<Utc>>>,
}

impl UpdateOneTokenReqJson {
    pub fn collection_rules(&self) -> &Option<HashMap<Uuid, TokenCollectionRuleMethodJson>> {
        &self.collection_rules
    }

    pub fn expired_at(&self) -> &Option<Option<DateTime<Utc>>> {
        &self.expired_at
    }

    pub fn is_all_none(&self) -> bool {
        self.collection_rules.is_none() && self.expired_at.is_none()
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
    bucket_rules: HashMap<Uuid, TokenBucketRuleMethodJson>,
    collection_rules: HashMap<Uuid, TokenCollectionRuleMethodJson>,
    expired_at: Option<DateTime<Utc>>,
}

impl TokenResJson {
    pub fn new(
        id: &Uuid,
        created_at: &DateTime<Utc>,
        updated_at: &DateTime<Utc>,
        token: &str,
        bucket_rules: &HashMap<Uuid, TokenBucketRuleMethodJson>,
        collection_rules: &HashMap<Uuid, TokenCollectionRuleMethodJson>,
        expired_at: &Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id: *id,
            created_at: *created_at,
            updated_at: *updated_at,
            token: token.to_owned(),
            bucket_rules: bucket_rules.clone(),
            collection_rules: collection_rules.clone(),
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
pub struct TokenBucketRuleMethodJson {
    find_one: Option<bool>,
    find_many: Option<bool>,
    insert: Option<bool>,
    update: Option<bool>,
    delete: Option<bool>,
    download_one: Option<bool>,
}

impl TokenBucketRuleMethodJson {
    pub fn from_dao(dao: &TokenBucketRuleMethod) -> Result<Self> {
        Ok(Self {
            find_one: Some(*dao.find_one()),
            find_many: Some(*dao.find_many()),
            insert: Some(*dao.insert()),
            update: Some(*dao.update()),
            delete: Some(*dao.delete()),
            download_one: Some(*dao.download_one()),
        })
    }

    pub fn to_dao(&self) -> Result<TokenBucketRuleMethod> {
        Ok(TokenBucketRuleMethod::new(
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
            &match self.download_one {
                Some(download_one) => download_one,
                None => false,
            },
        ))
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct TokenCollectionRuleMethodJson {
    find_one: Option<bool>,
    find_many: Option<bool>,
    insert: Option<bool>,
    update: Option<bool>,
    delete: Option<bool>,
}

impl TokenCollectionRuleMethodJson {
    pub fn from_dao(dao: &TokenCollectionRuleMethod) -> Result<Self> {
        Ok(Self {
            find_one: Some(*dao.find_one()),
            find_many: Some(*dao.find_many()),
            insert: Some(*dao.insert()),
            update: Some(*dao.update()),
            delete: Some(*dao.delete()),
        })
    }

    pub fn to_dao(&self) -> Result<TokenCollectionRuleMethod> {
        Ok(TokenCollectionRuleMethod::new(
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
