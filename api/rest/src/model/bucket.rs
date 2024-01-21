use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct InsertOneBucketReqPath {
    project_id: Uuid,
}

impl InsertOneBucketReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }
}

#[derive(Deserialize)]
pub struct InsertOneBucketReqJson {
    name: String,
}

impl InsertOneBucketReqJson {
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Deserialize)]
pub struct FindOneBucketReqPath {
    project_id: Uuid,
    bucket_id: Uuid,
}

impl FindOneBucketReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn bucket_id(&self) -> &Uuid {
        &self.bucket_id
    }
}

#[derive(Deserialize)]
pub struct UpdateOneBucketReqPath {
    project_id: Uuid,
    bucket_id: Uuid,
}

impl UpdateOneBucketReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn bucket_id(&self) -> &Uuid {
        &self.bucket_id
    }
}

#[derive(Deserialize)]
pub struct UpdateOneBucketReqJson {
    name: Option<String>,
}

impl UpdateOneBucketReqJson {
    pub fn name(&self) -> &Option<String> {
        &self.name
    }

    pub fn is_all_none(&self) -> bool {
        self.name.is_none()
    }
}

#[derive(Deserialize)]
pub struct DeleteOneBucketReqPath {
    project_id: Uuid,
    bucket_id: Uuid,
}

impl DeleteOneBucketReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn bucket_id(&self) -> &Uuid {
        &self.bucket_id
    }
}

#[derive(Deserialize)]
pub struct FindManyBucketReqPath {
    project_id: Uuid,
}

impl FindManyBucketReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }
}

#[derive(Serialize)]
pub struct BucketResJson {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    project_id: Uuid,
    name: String,
}

impl BucketResJson {
    pub fn new(
        id: &Uuid,
        created_at: &DateTime<Utc>,
        updated_at: &DateTime<Utc>,
        project_id: &Uuid,
        name: &str,
    ) -> Self {
        Self {
            id: *id,
            created_at: *created_at,
            updated_at: *updated_at,
            project_id: *project_id,
            name: name.to_owned(),
        }
    }
}

#[derive(Serialize)]
pub struct DeleteBucketResJson {
    id: Uuid,
}

impl DeleteBucketResJson {
    pub fn new(id: &Uuid) -> Self {
        Self { id: *id }
    }
}
