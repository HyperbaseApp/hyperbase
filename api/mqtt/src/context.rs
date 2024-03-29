use std::sync::Arc;

use hb_api_websocket::broadcaster::WebSocketBroadcaster;
use hb_dao::Db;

pub struct ApiMqttCtx {
    dao: ApiMqttDaoCtx,
    websocket: ApiMqttWsCtx,
}

impl ApiMqttCtx {
    pub fn new(dao: ApiMqttDaoCtx, websocket: ApiMqttWsCtx) -> Self {
        Self { dao, websocket }
    }

    pub fn dao(&self) -> &ApiMqttDaoCtx {
        &self.dao
    }

    pub fn websocket(&self) -> &ApiMqttWsCtx {
        &self.websocket
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

pub struct ApiMqttWsCtx {
    broadcaster: WebSocketBroadcaster,
}

impl ApiMqttWsCtx {
    pub fn new(broadcaster: WebSocketBroadcaster) -> Self {
        Self { broadcaster }
    }

    pub fn broadcaster(&self) -> &WebSocketBroadcaster {
        &self.broadcaster
    }
}
