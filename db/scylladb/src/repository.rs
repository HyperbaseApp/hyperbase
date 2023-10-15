use hb_db::DbRepository;
use scylla::Session;

pub struct ScyllaRepository {
    pub(super) session: Session,
}

impl DbRepository for ScyllaRepository {
    fn insert_one(&self) {}

    fn insert_many(&self) {}

    fn find_one(&self) {}

    fn find_many(&self) {}

    fn update_one(&self) {}

    fn update_many(&self) {}

    fn delete_one(&self) {}

    fn delete_many(&self) {}
}
