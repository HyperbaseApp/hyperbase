use crate::Driver;

pub trait BaseModel {
    fn driver(&self) -> &Driver;

    fn id(&self) -> &uuid::Uuid;
    fn created_at(&self) -> &std::time::Duration;
    fn updated_at(&self) -> &std::time::Duration;
}
