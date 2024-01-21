use ahash::HashMap;
use serde::{Deserialize, Serialize};
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
    bucket_rules: Json<HashMap<Uuid, TokenBucketRuleMethodModel>>,
    collection_rules: Json<HashMap<Uuid, TokenCollectionRuleMethodModel>>,
    expired_at: Option<DateTime<Utc>>,
}

impl TokenModel {
    pub fn new(
        id: &Uuid,
        created_at: &DateTime<Utc>,
        updated_at: &DateTime<Utc>,
        admin_id: &Uuid,
        token: &str,
        bucket_rules: &Json<HashMap<Uuid, TokenBucketRuleMethodModel>>,
        collection_rules: &Json<HashMap<Uuid, TokenCollectionRuleMethodModel>>,
        expired_at: &Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id: *id,
            created_at: *created_at,
            updated_at: *updated_at,
            admin_id: *admin_id,
            token: token.to_owned(),
            bucket_rules: bucket_rules.clone(),
            collection_rules: collection_rules.clone(),
            expired_at: *expired_at,
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

    pub fn bucket_rules(&self) -> &Json<HashMap<Uuid, TokenBucketRuleMethodModel>> {
        &self.bucket_rules
    }

    pub fn collection_rules(&self) -> &Json<HashMap<Uuid, TokenCollectionRuleMethodModel>> {
        &self.collection_rules
    }

    pub fn expired_at(&self) -> &Option<DateTime<Utc>> {
        &self.expired_at
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct TokenCollectionRuleMethodModel {
    find_one: bool,
    find_many: bool,
    insert: bool,
    update: bool,
    delete: bool,
}

impl TokenCollectionRuleMethodModel {
    pub fn new(
        find_one: &bool,
        find_many: &bool,
        insert: &bool,
        update: &bool,
        delete: &bool,
    ) -> Self {
        Self {
            find_one: *find_one,
            find_many: *find_many,
            insert: *insert,
            update: *update,
            delete: *delete,
        }
    }

    pub fn find_one(&self) -> &bool {
        &self.find_one
    }

    pub fn find_many(&self) -> &bool {
        &self.find_many
    }

    pub fn insert(&self) -> &bool {
        &self.insert
    }

    pub fn update(&self) -> &bool {
        &self.update
    }

    pub fn delete(&self) -> &bool {
        &self.delete
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct TokenBucketRuleMethodModel {
    find_one: bool,
    find_many: bool,
    insert: bool,
    update: bool,
    delete: bool,
    download_one: bool,
}

impl TokenBucketRuleMethodModel {
    pub fn new(
        find_one: &bool,
        find_many: &bool,
        insert: &bool,
        update: &bool,
        delete: &bool,
        download_one: &bool,
    ) -> Self {
        Self {
            find_one: *find_one,
            find_many: *find_many,
            insert: *insert,
            update: *update,
            delete: *delete,
            download_one: *download_one,
        }
    }

    pub fn find_one(&self) -> &bool {
        &self.find_one
    }

    pub fn find_many(&self) -> &bool {
        &self.find_many
    }

    pub fn insert(&self) -> &bool {
        &self.insert
    }

    pub fn update(&self) -> &bool {
        &self.update
    }

    pub fn delete(&self) -> &bool {
        &self.delete
    }

    pub fn download_one(&self) -> &bool {
        &self.download_one
    }
}
