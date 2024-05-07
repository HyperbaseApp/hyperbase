use std::sync::Arc;

use hb_dao::Db;

pub struct ApiWebSocketCtx {
    db: Arc<Db>,
}

impl ApiWebSocketCtx {
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &Db {
        &self.db
    }
}
