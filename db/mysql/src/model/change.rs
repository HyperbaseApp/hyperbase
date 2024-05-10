use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(FromRow)]
pub struct ChangeModel {
    table: String,
    id: Uuid,
    state: String,
    timestamp: DateTime<Utc>,
    change_id: Uuid,
}

impl ChangeModel {
    pub fn new(
        table: &str,
        id: &Uuid,
        state: &str,
        timestamp: &DateTime<Utc>,
        change_id: &Uuid,
    ) -> Self {
        Self {
            table: table.to_owned(),
            id: *id,
            state: state.to_owned(),
            timestamp: *timestamp,
            change_id: *change_id,
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

    pub fn timestamp(&self) -> &DateTime<Utc> {
        &self.timestamp
    }

    pub fn change_id(&self) -> &Uuid {
        &self.change_id
    }
}
