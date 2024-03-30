use sqlx::{
    prelude::FromRow,
    types::chrono::{DateTime, Utc},
};
use uuid::Uuid;

#[derive(FromRow)]
pub struct FileModel {
    id: Uuid,
    created_by: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    bucket_id: Uuid,
    file_name: String,
    content_type: String,
    size: i64,
}

impl FileModel {
    pub fn new(
        id: &Uuid,
        created_by: &Uuid,
        created_at: &DateTime<Utc>,
        updated_at: &DateTime<Utc>,
        bucket_id: &Uuid,
        file_name: &str,
        content_type: &str,
        size: &i64,
    ) -> Self {
        Self {
            id: *id,
            created_by: *created_by,
            created_at: *created_at,
            updated_at: *updated_at,
            bucket_id: *bucket_id,
            file_name: file_name.to_owned(),
            content_type: content_type.to_owned(),
            size: *size,
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn created_by(&self) -> &Uuid {
        &self.created_by
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }

    pub fn bucket_id(&self) -> &Uuid {
        &self.bucket_id
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    pub fn content_type(&self) -> &str {
        &self.content_type
    }

    pub fn size(&self) -> &i64 {
        &self.size
    }
}
