use scylla::{frame::value::CqlTimestamp, FromRow, SerializeRow};
use uuid::Uuid;

#[derive(FromRow, SerializeRow)]
pub struct BucketRuleModel {
    id: Uuid,
    created_at: CqlTimestamp,
    updated_at: CqlTimestamp,
    project_id: Uuid,
    token_id: Uuid,
    bucket_id: Uuid,
    find_one: bool,
    find_many: bool,
    insert_one: bool,
    update_one: bool,
    delete_one: bool,
}

impl BucketRuleModel {
    pub fn new(
        id: &Uuid,
        created_at: &CqlTimestamp,
        updated_at: &CqlTimestamp,
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

    pub fn token_id(&self) -> &Uuid {
        &self.token_id
    }

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
