use std::sync::Arc;

use hb_dao::Db;

pub struct ApiMqttCtx {
    dao: DaoCtx,
}

impl ApiMqttCtx {
    pub fn new(dao: DaoCtx) -> Self {
        Self { dao }
    }

    pub fn dao(&self) -> &DaoCtx {
        &self.dao
    }
}

pub struct DaoCtx {
    db: Arc<Db>,
}

impl DaoCtx {
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &Db {
        &self.db
    }
}
