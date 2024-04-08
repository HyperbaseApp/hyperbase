use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize)]
pub struct InsertOneProjectReqJson {
    name: String,
}

impl InsertOneProjectReqJson {
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Deserialize)]
pub struct FindOneProjectReqPath {
    project_id: Uuid,
}

impl FindOneProjectReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }
}

#[derive(Deserialize)]
pub struct UpdateOneProjectReqPath {
    project_id: Uuid,
}

impl UpdateOneProjectReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }
}

#[derive(Deserialize)]
pub struct UpdateOneProjectReqJson {
    name: Option<String>,
}

impl UpdateOneProjectReqJson {
    pub fn name(&self) -> &Option<String> {
        &self.name
    }

    pub fn is_all_none(&self) -> bool {
        self.name.is_none()
    }
}

#[derive(Deserialize)]
pub struct DeleteOneProjectReqPath {
    project_id: Uuid,
}

impl DeleteOneProjectReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }
}

#[derive(Deserialize)]
pub struct TransferOneProjectReqPath {
    project_id: Uuid,
}

impl TransferOneProjectReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }
}

#[derive(Deserialize, Validate)]
pub struct TransferOneProjectReqJson {
    #[validate(email)]
    admin_email: String,
}

impl TransferOneProjectReqJson {
    pub fn admin_email(&self) -> &str {
        &self.admin_email
    }
}

#[derive(Deserialize)]
pub struct DuplicateOneProjectReqPath {
    project_id: Uuid,
}

impl DuplicateOneProjectReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }
}

#[derive(Deserialize)]
pub struct DuplicateOneProjectReqJson {
    with_records: bool,
    with_files: bool,
}

impl DuplicateOneProjectReqJson {
    pub fn with_records(&self) -> &bool {
        &self.with_records
    }

    pub fn with_files(&self) -> &bool {
        &self.with_files
    }
}

#[derive(Serialize)]
pub struct ProjectResJson {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    name: String,
}

impl ProjectResJson {
    pub fn new(
        id: &Uuid,
        created_at: &DateTime<Utc>,
        updated_at: &DateTime<Utc>,
        name: &str,
    ) -> Self {
        Self {
            id: *id,
            created_at: *created_at,
            updated_at: *updated_at,
            name: name.to_owned(),
        }
    }
}

#[derive(Serialize)]
pub struct ProjectIDResJson {
    id: Uuid,
}

impl ProjectIDResJson {
    pub fn new(id: &Uuid) -> Self {
        Self { id: *id }
    }
}
