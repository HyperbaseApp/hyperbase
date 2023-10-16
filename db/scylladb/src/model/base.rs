use std::time::Duration;

use hb_db::{model::BaseModel as DbBaseModel, Driver as DbDriver};
use uuid::Uuid;

pub struct BaseModel {
    id: Uuid,
    created_at: Duration,
    updated_at: Duration,
}

impl DbBaseModel for BaseModel {
    fn driver(&self) -> &DbDriver {
        &DbDriver::Scylla
    }

    fn id(&self) -> &Uuid {
        &self.id
    }

    fn created_at(&self) -> &Duration {
        &self.created_at
    }

    fn updated_at(&self) -> &Duration {
        &self.updated_at
    }
}
