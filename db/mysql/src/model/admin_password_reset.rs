use sqlx::{
    types::chrono::{DateTime, Utc},
    FromRow,
};
use uuid::Uuid;

#[derive(FromRow)]
pub struct AdminPasswordResetModel {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    admin_id: Uuid,
    code: String,
}

impl AdminPasswordResetModel {
    pub fn new(
        id: &Uuid,
        created_at: &DateTime<Utc>,
        updated_at: &DateTime<Utc>,
        admin_id: &Uuid,
        code: &str,
    ) -> Self {
        Self {
            id: *id,
            created_at: *created_at,
            updated_at: *updated_at,
            admin_id: *admin_id,
            code: code.to_owned(),
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }

    pub fn admin_id(&self) -> &Uuid {
        &self.admin_id
    }

    pub fn code(&self) -> &str {
        &self.code
    }
}
