use hb_db::Repository;
use scylla::Session;

pub struct ScyllaRepository {
    pub session: Session,
}

impl Repository for ScyllaRepository {
    fn insert_one(&self) {}

    fn insert_many(&self) {}

    fn find_one(&self) {}

    fn find_many(&self) {}

    fn update_one(&self) {}

    fn update_many(&self) {}

    fn delete_one(&self) {}

    fn delete_many(&self) {}
}
