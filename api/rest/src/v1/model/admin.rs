use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct UpdateOneAdminReqJson {
    // #[validate(email)]
    // email: Option<String>,
    password: Option<String>,
}

impl UpdateOneAdminReqJson {
    // pub fn email(&self) -> &Option<String> {
    //     &self.email
    // }

    pub fn password(&self) -> &Option<String> {
        &self.password
    }
}

#[derive(Serialize)]
pub struct AdminResJson {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    email: String,
}

impl AdminResJson {
    pub fn new(
        id: &Uuid,
        created_at: &DateTime<Utc>,
        updated_at: &DateTime<Utc>,
        email: &str,
    ) -> Self {
        Self {
            id: *id,
            created_at: *created_at,
            updated_at: *updated_at,
            email: email.to_string(),
        }
    }
}

#[derive(Serialize)]
pub struct DeleteOneAdminResJson {
    id: Uuid,
}

impl DeleteOneAdminResJson {
    pub fn new(id: &Uuid) -> Self {
        Self { id: *id }
    }
}
