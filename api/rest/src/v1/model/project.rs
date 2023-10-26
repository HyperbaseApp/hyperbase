use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct InsertOneProjectReqPath {
    admin_id: Uuid,
}

impl InsertOneProjectReqPath {
    pub fn admin_id(&self) -> &Uuid {
        &self.admin_id
    }
}

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
    admin_id: Uuid,
    project_id: Uuid,
}

impl FindOneProjectReqPath {
    pub fn admin_id(&self) -> &Uuid {
        &self.admin_id
    }

    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }
}

#[derive(Deserialize)]
pub struct UpdateOneProjectReqPath {
    admin_id: Uuid,
    project_id: Uuid,
}

impl UpdateOneProjectReqPath {
    pub fn admin_id(&self) -> &Uuid {
        &self.admin_id
    }

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

#[derive(Deserialize)]
pub struct DeleteOneProjectReqPath {
    admin_id: Uuid,
    project_id: Uuid,
}

impl DeleteOneProjectReqPath {
    pub fn admin_id(&self) -> &Uuid {
        &self.admin_id
    }

    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }
}

#[derive(Deserialize)]
pub struct FindManyProjectReqPath {
    admin_id: Uuid,
}

impl FindManyProjectReqPath {
    pub fn admin_id(&self) -> &Uuid {
        &self.admin_id
    }
}
