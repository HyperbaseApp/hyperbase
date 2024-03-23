use sqlx::{
    types::chrono::{DateTime, Utc},
    FromRow,
};
use uuid::Uuid;

#[derive(FromRow)]
pub struct TokenModel {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    project_id: Uuid,
    admin_id: Uuid,
    name: String,
    token: String,
    allow_anonymous: bool,
    expired_at: Option<DateTime<Utc>>,
}

impl TokenModel {
    pub fn new(
        id: &Uuid,
        created_at: &DateTime<Utc>,
        updated_at: &DateTime<Utc>,
        project_id: &Uuid,
        admin_id: &Uuid,
        name: &str,
        token: &str,
        allow_anonymous: &bool,
        expired_at: &Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id: *id,
            created_at: *created_at,
            updated_at: *updated_at,
            project_id: *project_id,
            admin_id: *admin_id,
            name: name.to_owned(),
            token: token.to_owned(),
            allow_anonymous: *allow_anonymous,
            expired_at: *expired_at,
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

    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn admin_id(&self) -> &Uuid {
        &self.admin_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn allow_anonymous(&self) -> &bool {
        &self.allow_anonymous
    }

    pub fn expired_at(&self) -> &Option<DateTime<Utc>> {
        &self.expired_at
    }
}
