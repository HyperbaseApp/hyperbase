use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct InsertOneAdminJson {
    email: String,
    password: String,
}

impl InsertOneAdminJson {
    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn password(&self) -> &str {
        &self.password
    }
}

#[derive(Deserialize)]
pub struct FindOneAdminPath {
    admin_id: Uuid,
}

impl FindOneAdminPath {
    pub fn admin_id(&self) -> &Uuid {
        &self.admin_id
    }
}

#[derive(Deserialize)]
pub struct UpdateOneAdminPath {
    admin_id: Uuid,
}

impl UpdateOneAdminPath {
    pub fn admin_id(&self) -> &Uuid {
        &self.admin_id
    }
}

#[derive(Deserialize)]
pub struct UpdateOneAdminJson {
    email: Option<String>,
    password: Option<String>,
}

impl UpdateOneAdminJson {
    pub fn email(&self) -> &Option<String> {
        &self.email
    }

    pub fn password(&self) -> &Option<String> {
        &self.password
    }
}

#[derive(Deserialize)]
pub struct DeleteOneAdminPath {
    admin_id: Uuid,
}

impl DeleteOneAdminPath {
    pub fn admin_id(&self) -> &Uuid {
        &self.admin_id
    }
}
