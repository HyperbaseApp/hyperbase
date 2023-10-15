pub enum DbDriver {
    Scylla,
}

pub trait DbBaseModel {
    fn id(&self) -> &uuid::Uuid;
    fn created_at(&self) -> &std::time::Duration;
    fn updated_at(&self) -> &std::time::Duration;
}

pub trait DbCollection: DbBaseModel {
    fn driver(&self) -> &DbDriver;
    fn collection(&self) -> &str {
        "_collections"
    }
    fn migrate(&self);

    fn name(&self) -> &str;
}

pub trait DbRecord: DbBaseModel {
    fn collection(&self) -> &dyn DbCollection;
    fn schema(&self) -> &std::collections::HashMap<&str, Box<dyn DbRecordSchemaV>>;
}

pub trait DbRecordSchemaV {
    fn kind(&self) -> &str;
    fn value(&self) -> &Box<dyn std::any::Any>;
}

pub trait DbRepository {
    fn insert_one(&self);
    fn insert_many(&self);
    fn find_one(&self);
    fn find_many(&self);
    fn update_one(&self);
    fn update_many(&self);
    fn delete_one(&self);
    fn delete_many(&self);
}
