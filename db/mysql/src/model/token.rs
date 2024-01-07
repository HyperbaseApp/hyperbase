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
    rules: Json<HashMap<Uuid, TokenRuleMethodModel>>,
    expired_at: Option<DateTime<Utc>>,
}

impl TokenModel {
    pub fn new(
        id: &Uuid,
        created_at: &DateTime<Utc>,
        updated_at: &DateTime<Utc>,
        admin_id: &Uuid,
        token: &str,
        rules: &Json<HashMap<Uuid, TokenRuleMethodModel>>,
        expired_at: &Option<DateTime<Utc>>,
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

    pub fn rules(&self) -> &Json<HashMap<Uuid, TokenRuleMethodModel>> {
        &self.rules
    }

    pub fn expired_at(&self) -> &Option<DateTime<Utc>> {
        &self.expired_at
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct TokenRuleMethodModel {
    find_one: bool,
    find_many: bool,
    insert: bool,
    update: bool,
    delete: bool,
}

impl TokenRuleMethodModel {
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
