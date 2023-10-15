use hb_db::{DbCollection, DbDriver, DbBaseModel};

use super::base::ScyllaBaseModel;

pub struct ScyllaCollection<'a> {
    base_model: ScyllaBaseModel,
    name: &'a str,
}

impl<'a> DbBaseModel for ScyllaCollection<'a> {
    fn id(&self) -> &uuid::Uuid {
        &self.base_model.id
    }

    fn created_at(&self) -> &std::time::Duration {
        &self.base_model.created_at
    }

    fn updated_at(&self) -> &std::time::Duration {
        &self.base_model.updated_at
    }
}

impl<'a> DbCollection for ScyllaCollection<'a> {
    fn driver(&self) -> &DbDriver {
        &DbDriver::Scylla
    }

    fn migrate(&self) {
        todo!("implement collection migration")
    }

    fn name(&self) -> &str {
        self.name
    }
}
