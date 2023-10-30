use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
}

impl UpdateOneProjectReqJson {
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

#[derive(Serialize)]
pub struct ProjectResJson {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    admin_id: Uuid,
    name: String,
}

impl ProjectResJson {
    pub fn new(
        id: &Uuid,
        created_at: &DateTime<Utc>,
        updated_at: &DateTime<Utc>,
        admin_id: &Uuid,
        name: &str,
    ) -> Self {
        Self {
            id: *id,
            created_at: *created_at,
            updated_at: *updated_at,
            admin_id: *admin_id,
            name: name.to_string(),
        }
    }
}

#[derive(Serialize)]
pub struct DeleteProjectResJson {
    id: Uuid,
}

impl DeleteProjectResJson {
    pub fn new(id: &Uuid) -> Self {
        Self { id: *id }
    }
}
