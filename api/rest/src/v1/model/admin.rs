use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize)]
pub struct FindOneAdminReqPath {
    admin_id: Uuid,
}

impl FindOneAdminReqPath {
    pub fn admin_id(&self) -> &Uuid {
        &self.admin_id
    }
}

#[derive(Deserialize)]
pub struct UpdateOneAdminReqPath {
    admin_id: Uuid,
}

impl UpdateOneAdminReqPath {
    pub fn admin_id(&self) -> &Uuid {
        &self.admin_id
    }
}

#[derive(Deserialize, Validate)]
pub struct UpdateOneAdminReqJson {
    #[validate(email)]
    email: Option<String>,
    password: Option<String>,
}

impl UpdateOneAdminReqJson {
    pub fn email(&self) -> &Option<String> {
        &self.email
    }

    pub fn password(&self) -> &Option<String> {
        &self.password
    }
}

#[derive(Deserialize)]
pub struct DeleteOneAdminReqPath {
    admin_id: Uuid,
}

impl DeleteOneAdminReqPath {
    pub fn admin_id(&self) -> &Uuid {
        &self.admin_id
    }
}
