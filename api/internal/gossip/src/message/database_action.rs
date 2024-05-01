use uuid::Uuid;

pub struct DatabaseActionMessage {
    kind: DatabaseUpdateKind,
    table: String,
    data_id: Uuid,
    change_id: Uuid,
    data_value: Vec<u8>,
}

impl DatabaseActionMessage {
    pub fn new(
        kind: DatabaseUpdateKind,
        table: String,
        data_id: Uuid,
        change_id: Uuid,
        data_value: Vec<u8>,
    ) -> Self {
        Self {
            kind,
            table,
            data_id,
            change_id,
            data_value,
        }
    }
}

pub enum DatabaseUpdateKind {
    Create,
    Update,
    Delete,
}
