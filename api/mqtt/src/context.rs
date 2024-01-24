use std::sync::Arc;

use hb_dao::Db;

pub struct ApiMqttCtx {
    dao: ApiMqttDaoCtx,
}

impl ApiMqttCtx {
    pub fn new(dao: ApiMqttDaoCtx) -> Self {
        Self { dao }
    }

    pub fn dao(&self) -> &ApiMqttDaoCtx {
        &self.dao
    }
}

pub struct ApiMqttDaoCtx {
    db: Arc<Db>,
}

impl ApiMqttDaoCtx {
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &Db {
        &self.db
    }
}
