use std::path::Path;

use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use chrono::{DateTime, Utc};
use mime::Mime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct InsertOneFileReqPath {
    project_id: Uuid,
    bucket_id: Uuid,
}

impl InsertOneFileReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn bucket_id(&self) -> &Uuid {
        &self.bucket_id
    }
}

#[derive(MultipartForm)]
pub struct InsertOneFileReqForm {
    file: TempFile,
    file_name: Option<Text<String>>,
}

impl InsertOneFileReqForm {
    pub fn file_path(&self) -> &Path {
        self.file.file.path()
    }

    pub fn file_name(&self) -> Option<String> {
        if let Some(name) = &self.file_name {
            Some(name.0.to_owned())
        } else if let Some(name) = &self.file.file_name {
            Some(name.to_owned())
        } else {
            None
        }
    }

    pub fn content_type(&self) -> &Option<Mime> {
        &self.file.content_type
    }

    pub fn size(&self) -> &usize {
        &self.file.size
    }
}

#[derive(Deserialize)]
pub struct FindOneFileReqPath {
    project_id: Uuid,
    bucket_id: Uuid,
    file_id: Uuid,
}

impl FindOneFileReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn bucket_id(&self) -> &Uuid {
        &self.bucket_id
    }

    pub fn file_id(&self) -> &Uuid {
        &self.file_id
    }
}

#[derive(Deserialize)]
pub struct FindOneFileReqQuery {
    token: String,
}

impl FindOneFileReqQuery {
    pub fn token(&self) -> &str {
        &self.token
    }
}

#[derive(Deserialize)]
pub struct UpdateOneFileReqPath {
    project_id: Uuid,
    bucket_id: Uuid,
    file_id: Uuid,
}

impl UpdateOneFileReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn bucket_id(&self) -> &Uuid {
        &self.bucket_id
    }

    pub fn file_id(&self) -> &Uuid {
        &self.file_id
    }
}

#[derive(Deserialize)]
pub struct UpdateOneFileReqJson {
    created_by: Option<Uuid>,
    file_name: Option<String>,
}

impl UpdateOneFileReqJson {
    pub fn created_by(&self) -> &Option<Uuid> {
        &self.created_by
    }

    pub fn file_name(&self) -> &Option<String> {
        &self.file_name
    }

    pub fn is_all_none(&self) -> bool {
        self.created_by.is_none() && self.file_name.is_none()
    }
}

#[derive(Deserialize)]
pub struct DeleteOneFileReqPath {
    project_id: Uuid,
    bucket_id: Uuid,
    file_id: Uuid,
}

impl DeleteOneFileReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn bucket_id(&self) -> &Uuid {
        &self.bucket_id
    }

    pub fn file_id(&self) -> &Uuid {
        &self.file_id
    }
}

#[derive(Deserialize)]
pub struct FindManyFileReqPath {
    project_id: Uuid,
    bucket_id: Uuid,
}

impl FindManyFileReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn bucket_id(&self) -> &Uuid {
        &self.bucket_id
    }
}

#[derive(Deserialize)]
pub struct FindManyFileReqQuery {
    after_id: Option<Uuid>,
    limit: Option<i32>,
}

impl FindManyFileReqQuery {
    pub fn after_id(&self) -> &Option<Uuid> {
        &self.after_id
    }

    pub fn limit(&self) -> &Option<i32> {
        &self.limit
    }
}

#[derive(Serialize)]
pub struct FileResJson {
    id: Uuid,
    created_by: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    bucket_id: Uuid,
    file_name: String,
    content_type: String,
    size: i64,
}

impl FileResJson {
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
}

#[derive(Serialize)]
pub struct DeleteFileResJson {
    id: Uuid,
}

impl DeleteFileResJson {
    pub fn new(id: &Uuid) -> Self {
        Self { id: *id }
    }
}
