use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(FromRow)]
pub struct LocalInfoModel {
    id: Uuid,
}

impl LocalInfoModel {
    pub fn new(id: &Uuid) -> Self {
        Self { id: *id }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }
}
