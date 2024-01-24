use std::sync::{mpsc::Sender, Arc};

use hb_dao::Db;
use hb_hash_argon2::argon2::Argon2Hash;
use hb_mailer::MailPayload;
use hb_token_jwt::token::JwtToken;

pub struct ApiRestCtx {
    hash: ApiRestHashCtx,
    token: ApiRestTokenCtx,
    mailer: ApiRestMailerCtx,
    dao: ApiRestDaoCtx,
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
        mailer: ApiRestMailerCtx,
        dao: ApiRestDaoCtx,
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

    pub fn mailer(&self) -> &ApiRestMailerCtx {
        &self.mailer
    }

    pub fn dao(&self) -> &ApiRestDaoCtx {
        &self.dao
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

    pub fn bucket_path(&self) -> &str {
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
    sender: Sender<MailPayload>,
}

impl ApiRestMailerCtx {
    pub fn new(sender: Sender<MailPayload>) -> Self {
        Self { sender }
    }

    pub fn sender(&self) -> &Sender<MailPayload> {
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
