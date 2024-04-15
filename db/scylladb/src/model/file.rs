use scylla::{frame::value::CqlTimestamp, FromRow, SerializeRow};
use uuid::Uuid;

#[derive(FromRow, SerializeRow)]
pub struct FileModel {
    id: Uuid,
    created_by: Uuid,
    created_at: CqlTimestamp,
    updated_at: CqlTimestamp,
    bucket_id: Uuid,
    file_name: String,
    content_type: String,
    size: i64,
    public: bool,
}

impl FileModel {
    pub fn new(
        id: &Uuid,
        created_by: &Uuid,
        created_at: &CqlTimestamp,
        updated_at: &CqlTimestamp,
        bucket_id: &Uuid,
        file_name: &String,
        content_type: &str,
        size: &i64,
        public: &bool,
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
            public: *public,
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn created_by(&self) -> &Uuid {
        &self.created_by
    }

    pub fn created_at(&self) -> &CqlTimestamp {
        &self.created_at
    }

    pub fn updated_at(&self) -> &CqlTimestamp {
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

    pub fn public(&self) -> &bool {
        &self.public
    }
}
