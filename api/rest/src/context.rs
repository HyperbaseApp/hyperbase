use std::sync::Arc;

use hb_api_websocket::handler::WebSocketHandler;
use hb_dao::Db;
use hb_hash_argon2::argon2::Argon2Hash;
use hb_mailer::MailPayload;
use hb_token_jwt::token::JwtToken;
use tokio::sync::mpsc;

pub struct ApiRestCtx {
    hash: ApiRestHashCtx,
    token: ApiRestTokenCtx,
    mailer: Option<ApiRestMailerCtx>,
    dao: ApiRestDaoCtx,
    websocket: ApiRestWsCtx,
    mqtt_admin_credential: Option<MqttAdminCredential>,
    admin_registration: bool,
    access_token_length: usize,
    registration_ttl: u32,
    reset_password_ttl: u32,
    bucket_path: String,
}

impl ApiRestCtx {
    pub fn new(
        hash: ApiRestHashCtx,
        token: ApiRestTokenCtx,
        mailer: Option<ApiRestMailerCtx>,
        dao: ApiRestDaoCtx,
        websocket: ApiRestWsCtx,
        mqtt_admin_credential: Option<MqttAdminCredential>,
        admin_registration: bool,
        access_token_length: usize,
        registration_ttl: u32,
        reset_password_ttl: u32,
        bucket_path: String,
    ) -> Self {
        Self {
            hash,
            token,
            mailer,
            dao,
            websocket,
            mqtt_admin_credential,
            admin_registration,
            access_token_length,
            registration_ttl,
            reset_password_ttl,
            bucket_path,
        }
    }

    pub fn hash(&self) -> &ApiRestHashCtx {
        &self.hash
    }

    pub fn token(&self) -> &ApiRestTokenCtx {
        &self.token
    }

    pub fn mailer(&self) -> &Option<ApiRestMailerCtx> {
        &self.mailer
    }

    pub fn dao(&self) -> &ApiRestDaoCtx {
        &self.dao
    }

    pub fn websocket(&self) -> &ApiRestWsCtx {
        &self.websocket
    }

    pub fn mqtt_admin_credential(&self) -> &Option<MqttAdminCredential> {
        &self.mqtt_admin_credential
    }

    pub fn admin_registration(&self) -> &bool {
        &self.admin_registration
    }

    pub fn access_token_length(&self) -> &usize {
        &self.access_token_length
    }

    pub fn registration_ttl(&self) -> &u32 {
        &self.registration_ttl
    }

    pub fn reset_password_ttl(&self) -> &u32 {
        &self.reset_password_ttl
    }

    pub fn bucket_path(&self) -> &String {
        &self.bucket_path
    }
}

pub struct ApiRestHashCtx {
    argon2: Argon2Hash,
}

impl ApiRestHashCtx {
    pub fn new(argon2: Argon2Hash) -> Self {
        Self { argon2 }
    }

    pub fn argon2(&self) -> &Argon2Hash {
        &self.argon2
    }
}

pub struct ApiRestTokenCtx {
    jwt: JwtToken,
}

impl ApiRestTokenCtx {
    pub fn new(jwt: JwtToken) -> Self {
        Self { jwt }
    }

    pub fn jwt(&self) -> &JwtToken {
        &self.jwt
    }
}

pub struct ApiRestMailerCtx {
    sender: mpsc::Sender<MailPayload>,
}

impl ApiRestMailerCtx {
    pub fn new(sender: mpsc::Sender<MailPayload>) -> Self {
        Self { sender }
    }

    pub fn sender(&self) -> &mpsc::Sender<MailPayload> {
        &self.sender
    }
}

pub struct ApiRestDaoCtx {
    db: Arc<Db>,
}

impl ApiRestDaoCtx {
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &Db {
        &self.db
    }
}

pub struct ApiRestWsCtx {
    handler: WebSocketHandler,
}

impl ApiRestWsCtx {
    pub fn new(handler: WebSocketHandler) -> Self {
        Self { handler }
    }

    pub fn handler(&self) -> &WebSocketHandler {
        &self.handler
    }
}

pub struct MqttAdminCredential {
    username: String,
    password: String,
    topic: String,
}

impl MqttAdminCredential {
    pub fn new(username: &str, password: &str, topic: &str) -> Self {
        Self {
            username: username.to_owned(),
            password: password.to_owned(),
            topic: topic.to_owned(),
        }
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn topic(&self) -> &str {
        &self.topic
    }
}
