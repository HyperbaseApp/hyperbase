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
    public: Option<Text<bool>>,
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

    pub fn public(&self) -> bool {
        match &self.public {
            Some(public) => public.0,
            None => false,
        }
    }
}

#[derive(Deserialize)]
pub struct HeadFindOneFileReqPath {
    project_id: Uuid,
    bucket_id: Uuid,
    file_id: Uuid,
}

impl HeadFindOneFileReqPath {
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
pub struct HeadFindOneFileReqQuery {
    token: Option<String>,
}

impl HeadFindOneFileReqQuery {
    pub fn token(&self) -> &Option<String> {
        &self.token
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
    token: Option<String>,
    data: Option<u8>,
}

impl FindOneFileReqQuery {
    pub fn token(&self) -> &Option<String> {
        &self.token
    }

    pub fn data(&self) -> &Option<u8> {
        &self.data
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
    public: Option<bool>,
}

impl UpdateOneFileReqJson {
    pub fn created_by(&self) -> &Option<Uuid> {
        &self.created_by
    }

    pub fn file_name(&self) -> &Option<String> {
        &self.file_name
    }

    pub fn public(&self) -> &Option<bool> {
        &self.public
    }

    pub fn is_all_none(&self) -> bool {
        self.created_by.is_none() && self.file_name.is_none() && self.public.is_none()
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
    before_id: Option<Uuid>,
    limit: Option<i32>,
}

impl FindManyFileReqQuery {
    pub fn before_id(&self) -> &Option<Uuid> {
        &self.before_id
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
    public: bool,
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
