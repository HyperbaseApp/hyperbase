use std::sync::Arc;

use hb_dao::Db;

pub struct ApiWebSocketCtx {
    dao: ApiWebSocketDaoCtx,
}

impl ApiWebSocketCtx {
    pub fn new(dao: ApiWebSocketDaoCtx) -> Self {
        Self { dao }
    }

    pub fn dao(&self) -> &ApiWebSocketDaoCtx {
        &self.dao
    }
}

pub struct ApiWebSocketDaoCtx {
    db: Arc<Db>,
}

impl ApiWebSocketDaoCtx {
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &Db {
        &self.db
    }
}
