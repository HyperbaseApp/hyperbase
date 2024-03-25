use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct InsertOneBucketRuleReqPath {
    project_id: Uuid,
    token_id: Uuid,
}

impl InsertOneBucketRuleReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn token_id(&self) -> &Uuid {
        &self.token_id
    }
}

#[derive(Deserialize)]
pub struct InsertOneBucketRuleReqJson {
    bucket_id: Uuid,
    find_one: bool,
    find_many: bool,
    insert_one: bool,
    update_one: bool,
    delete_one: bool,
}

impl InsertOneBucketRuleReqJson {
    pub fn bucket_id(&self) -> &Uuid {
        &self.bucket_id
    }

    pub fn find_one(&self) -> &bool {
        &self.find_one
    }

    pub fn find_many(&self) -> &bool {
        &self.find_many
    }

    pub fn insert_one(&self) -> &bool {
        &self.insert_one
    }

    pub fn update_one(&self) -> &bool {
        &self.update_one
    }

    pub fn delete_one(&self) -> &bool {
        &self.delete_one
    }
}

#[derive(Deserialize)]
pub struct FindOneBucketRuleReqPath {
    token_id: Uuid,
    rule_id: Uuid,
}

impl FindOneBucketRuleReqPath {
    pub fn token_id(&self) -> &Uuid {
        &self.token_id
    }

    pub fn rule_id(&self) -> &Uuid {
        &self.rule_id
    }
}

#[derive(Deserialize)]
pub struct UpdateOneBucketRuleReqPath {
    token_id: Uuid,
    rule_id: Uuid,
}

impl UpdateOneBucketRuleReqPath {
    pub fn token_id(&self) -> &Uuid {
        &self.token_id
    }

    pub fn rule_id(&self) -> &Uuid {
        &self.rule_id
    }
}

#[derive(Deserialize)]
pub struct UpdateOneBucketRuleReqJson {
    find_one: Option<bool>,
    find_many: Option<bool>,
    insert_one: Option<bool>,
    update_one: Option<bool>,
    delete_one: Option<bool>,
}

impl UpdateOneBucketRuleReqJson {
    pub fn find_one(&self) -> &Option<bool> {
        &self.find_one
    }

    pub fn find_many(&self) -> &Option<bool> {
        &self.find_many
    }

    pub fn insert_one(&self) -> &Option<bool> {
        &self.insert_one
    }

    pub fn update_one(&self) -> &Option<bool> {
        &self.update_one
    }

    pub fn delete_one(&self) -> &Option<bool> {
        &self.delete_one
    }

    pub fn is_all_none(&self) -> bool {
        self.find_one.is_none()
            && self.find_many.is_none()
            && self.insert_one.is_none()
            && self.update_one.is_none()
            && self.delete_one.is_none()
    }
}

#[derive(Deserialize)]
pub struct DeleteOneBucketRuleReqPath {
    token_id: Uuid,
    rule_id: Uuid,
}

impl DeleteOneBucketRuleReqPath {
    pub fn token_id(&self) -> &Uuid {
        &self.token_id
    }

    pub fn rule_id(&self) -> &Uuid {
        &self.rule_id
    }
}

#[derive(Deserialize)]
pub struct FindManyBucketRuleReqPath {
    token_id: Uuid,
}

impl FindManyBucketRuleReqPath {
    pub fn token_id(&self) -> &Uuid {
        &self.token_id
    }
}

#[derive(Serialize)]
pub struct BucketRuleResJson {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    project_id: Uuid,
    token_id: Uuid,
    bucket_id: Uuid,
    find_one: bool,
    find_many: bool,
    insert_one: bool,
    update_one: bool,
    delete_one: bool,
}

impl BucketRuleResJson {
    pub fn new(
        id: &Uuid,
        created_at: &DateTime<Utc>,
        updated_at: &DateTime<Utc>,
        project_id: &Uuid,
        token_id: &Uuid,
        bucket_id: &Uuid,
        find_one: &bool,
        find_many: &bool,
        insert_one: &bool,
        update_one: &bool,
        delete_one: &bool,
    ) -> Self {
        Self {
            id: *id,
            created_at: *created_at,
            updated_at: *updated_at,
            project_id: *project_id,
            token_id: *token_id,
            bucket_id: *bucket_id,
            find_one: *find_one,
            find_many: *find_many,
            insert_one: *insert_one,
            update_one: *update_one,
            delete_one: *delete_one,
        }
    }
}

#[derive(Serialize)]
pub struct DeleteBucketRuleResJson {
    id: Uuid,
}

impl DeleteBucketRuleResJson {
    pub fn new(id: &Uuid) -> Self {
        Self { id: *id }
    }
}
