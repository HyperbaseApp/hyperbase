use sqlx::{
    prelude::FromRow,
    types::chrono::{DateTime, Utc},
};
use uuid::Uuid;

#[derive(FromRow)]
pub struct BucketModel {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    project_id: Uuid,
    name: String,
    path: String,
    opt_ttl: Option<i64>,
}

impl BucketModel {
    pub fn new(
        id: &Uuid,
        created_at: &DateTime<Utc>,
        updated_at: &DateTime<Utc>,
        project_id: &Uuid,
        name: &str,
        path: &str,
        opt_ttl: &Option<i64>,
    ) -> Self {
        Self {
            id: *id,
            created_at: *created_at,
            updated_at: *updated_at,
            project_id: *project_id,
            name: name.to_owned(),
            path: path.to_owned(),
            opt_ttl: *opt_ttl,
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn opt_ttl(&self) -> &Option<i64> {
        &self.opt_ttl
    }
}
