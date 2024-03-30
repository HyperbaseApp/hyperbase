use scylla::{frame::value::CqlTimestamp, FromRow, SerializeRow};
use uuid::Uuid;

#[derive(FromRow, SerializeRow)]
pub struct CollectionRuleModel {
    id: Uuid,
    created_at: CqlTimestamp,
    updated_at: CqlTimestamp,
    project_id: Uuid,
    token_id: Uuid,
    collection_id: Uuid,
    find_one: String,
    find_many: String,
    insert_one: bool,
    update_one: String,
    delete_one: String,
}

impl CollectionRuleModel {
    pub fn new(
        id: &Uuid,
        created_at: &CqlTimestamp,
        updated_at: &CqlTimestamp,
        project_id: &Uuid,
        token_id: &Uuid,
        collection_id: &Uuid,
        find_one: &str,
        find_many: &str,
        insert_one: &bool,
        update_one: &str,
        delete_one: &str,
    ) -> Self {
        Self {
            id: *id,
            created_at: *created_at,
            updated_at: *updated_at,
            project_id: *project_id,
            token_id: *token_id,
            collection_id: *collection_id,
            find_one: find_one.to_owned(),
            find_many: find_many.to_owned(),
            insert_one: *insert_one,
            update_one: update_one.to_owned(),
            delete_one: delete_one.to_owned(),
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

    pub fn collection_id(&self) -> &Uuid {
        &self.collection_id
    }

    pub fn find_one(&self) -> &str {
        &self.find_one
    }

    pub fn find_many(&self) -> &str {
        &self.find_many
    }

    pub fn insert_one(&self) -> &bool {
        &self.insert_one
    }

    pub fn update_one(&self) -> &str {
        &self.update_one
    }

    pub fn delete_one(&self) -> &str {
        &self.delete_one
    }
}
