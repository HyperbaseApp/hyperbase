use ahash::HashMap;
use scylla::{frame::value::CqlTimestamp, FromRow, FromUserType, SerializeCql, SerializeRow};
use uuid::Uuid;

#[derive(FromRow, SerializeRow)]
pub struct TokenModel {
    id: Uuid,
    created_at: CqlTimestamp,
    updated_at: CqlTimestamp,
    project_id: Uuid,
    admin_id: Uuid,
    token: String,
    bucket_rules: Option<HashMap<Uuid, TokenBucketRuleMethodModel>>,
    collection_rules: Option<HashMap<Uuid, TokenCollectionRuleMethodModel>>,
    expired_at: Option<CqlTimestamp>,
}

impl TokenModel {
    pub fn new(
        id: &Uuid,
        created_at: &CqlTimestamp,
        updated_at: &CqlTimestamp,
        project_id: &Uuid,
        admin_id: &Uuid,
        token: &str,
        bucket_rules: &Option<HashMap<Uuid, TokenBucketRuleMethodModel>>,
        collection_rules: &Option<HashMap<Uuid, TokenCollectionRuleMethodModel>>,
        expired_at: &Option<CqlTimestamp>,
    ) -> Self {
        Self {
            id: *id,
            created_at: *created_at,
            updated_at: *updated_at,
            project_id: *project_id,
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

    pub fn created_at(&self) -> &CqlTimestamp {
        &self.created_at
    }

    pub fn updated_at(&self) -> &CqlTimestamp {
        &self.updated_at
    }

    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn admin_id(&self) -> &Uuid {
        &self.admin_id
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn bucket_rules(&self) -> &Option<HashMap<Uuid, TokenBucketRuleMethodModel>> {
        &self.bucket_rules
    }

    pub fn collection_rules(&self) -> &Option<HashMap<Uuid, TokenCollectionRuleMethodModel>> {
        &self.collection_rules
    }

    pub fn expired_at(&self) -> &Option<CqlTimestamp> {
        &self.expired_at
    }
}

#[derive(FromUserType, SerializeCql, Clone)]
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

#[derive(FromUserType, SerializeCql, Clone)]
pub struct TokenBucketRuleMethodModel {
    find_one: bool,
    find_many: bool,
    insert: bool,
    update: bool,
    delete: bool,
}

impl TokenBucketRuleMethodModel {
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
