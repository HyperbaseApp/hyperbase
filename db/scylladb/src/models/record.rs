use hb_db::{DbBaseModel, DbCollection, DbRecord, DbRecordSchemaV};

use super::{base::ScyllaBaseModel, collection::ScyllaCollection};

pub struct ScyllaRecord<'a> {
    base_model: ScyllaBaseModel,
    collection: ScyllaCollection<'a>,
    schema: std::collections::HashMap<&'a str, Box<dyn DbRecordSchemaV>>,
}

impl<'a> DbBaseModel for ScyllaRecord<'a> {
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

impl<'a> DbRecord for ScyllaRecord<'a> {
    fn collection(&self) -> &dyn DbCollection {
        &self.collection
    }

    fn schema(&self) -> &std::collections::HashMap<&str, Box<dyn hb_db::DbRecordSchemaV>> {
        &self.schema
    }
}
