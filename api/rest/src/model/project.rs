use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct InsertOneProjectPath {
    admin_id: Uuid,
}

impl InsertOneProjectPath {
    pub fn admin_id(&self) -> &Uuid {
        &self.admin_id
    }
}

#[derive(Deserialize)]
pub struct InsertOneProjectJson {
    name: String,
}

impl InsertOneProjectJson {
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Deserialize)]
pub struct FindOneProjectPath {
    admin_id: Uuid,
    project_id: Uuid,
}

impl FindOneProjectPath {
    pub fn admin_id(&self) -> &Uuid {
        &self.admin_id
    }

    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }
}

#[derive(Deserialize)]
pub struct UpdateOneProjectPath {
    admin_id: Uuid,
    project_id: Uuid,
}

impl UpdateOneProjectPath {
    pub fn admin_id(&self) -> &Uuid {
        &self.admin_id
    }

    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }
}

#[derive(Deserialize)]
pub struct UpdateOneProjectJson {
    name: Option<String>,
}

impl UpdateOneProjectJson {
    pub fn name(&self) -> &Option<String> {
        &self.name
    }
}

#[derive(Deserialize)]
pub struct DeleteOneProjectPath {
    admin_id: Uuid,
    project_id: Uuid,
}

impl DeleteOneProjectPath {
    pub fn admin_id(&self) -> &Uuid {
        &self.admin_id
    }

    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }
}

#[derive(Deserialize)]
pub struct FindManyProjectPath {
    admin_id: Uuid,
}

impl FindManyProjectPath {
    pub fn admin_id(&self) -> &Uuid {
        &self.admin_id
    }
}
