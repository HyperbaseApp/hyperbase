use scylla::{frame::value::CqlTimestamp, FromRow, SerializeRow};
use uuid::Uuid;

#[derive(FromRow, SerializeRow)]
pub struct ChangeModel {
    table: String,
    id: Uuid,
    state: String,
    updated_at: CqlTimestamp,
}

impl ChangeModel {
    pub fn new(table: &str, id: &Uuid, state: &str, updated_at: &CqlTimestamp) -> Self {
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

    pub fn updated_at(&self) -> &CqlTimestamp {
        &self.updated_at
    }
}
