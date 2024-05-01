use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(FromRow)]
pub struct ChangeModel {
    table: String,
    id: Uuid,
    state: String,
    updated_at: DateTime<Utc>,
}

impl ChangeModel {
    pub fn new(table: &str, id: &Uuid, state: &str, updated_at: &DateTime<Utc>) -> Self {
        Self {
            table: table.to_owned(),
            id: *id,
            state: state.to_owned(),
            updated_at: *updated_at,
        }
    }

    pub fn table(&self) -> &str {
        &self.table
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn state(&self) -> &str {
        &self.state
    }

    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }
}
