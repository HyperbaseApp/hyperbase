use scylla::frame::value::Timestamp;
use uuid::Uuid;

pub struct BaseScyllaModel {
    id: Uuid,
    created_at: Timestamp,
    updated_at: Timestamp,
}

impl BaseScyllaModel {
    fn id(&self) -> &Uuid {
        &self.id
    }

    fn created_at(&self) -> &Timestamp {
        &self.created_at
    }

    fn updated_at(&self) -> &Timestamp {
        &self.updated_at
    }
}
