use std::sync::mpsc::Sender;

use hb_dao::Db;
use hb_hash_argon2::argon2::Argon2Hash;
use hb_mailer::MailPayload;
use hb_token_jwt::token::JwtToken;

pub struct ApiRestCtx {
    hash: HashCtx,
    token: TokenCtx,
    mailer: MailerCtx,
    dao: DaoCtx,
    registration_ttl: i64,
    reset_password_ttl: i64,
    access_token_length: usize,
}

impl ApiRestCtx {
    pub fn new(
        hash: HashCtx,
        token: TokenCtx,
        mailer: MailerCtx,
        dao: DaoCtx,
        registration_ttl: i64,
        reset_password_ttl: i64,
        access_token_length: usize,
    ) -> Self {
        Self {
            hash,
            token,
            mailer,
            dao,
            registration_ttl,
            reset_password_ttl,
            access_token_length,
        }
    }

    pub fn hash(&self) -> &HashCtx {
        &self.hash
    }

    pub fn token(&self) -> &TokenCtx {
        &self.token
    }

    pub fn mailer(&self) -> &MailerCtx {
        &self.mailer
    }

    pub fn dao(&self) -> &DaoCtx {
        &self.dao
    }

    pub fn registration_ttl(&self) -> &i64 {
        &self.registration_ttl
    }

    pub fn reset_password_ttl(&self) -> &i64 {
        &self.reset_password_ttl
    }

    pub fn access_token_length(&self) -> &usize {
        &self.access_token_length
    }
}

pub struct HashCtx {
    argon2: Argon2Hash,
}

impl HashCtx {
    pub fn new(argon2: Argon2Hash) -> Self {
        Self { argon2 }
    }

    pub fn argon2(&self) -> &Argon2Hash {
        &self.argon2
    }
}

pub struct TokenCtx {
    jwt: JwtToken,
}

impl TokenCtx {
    pub fn new(jwt: JwtToken) -> Self {
        Self { jwt }
    }

    pub fn jwt(&self) -> &JwtToken {
        &self.jwt
    }
}

pub struct MailerCtx {
    sender: Sender<MailPayload>,
}

impl MailerCtx {
    pub fn new(sender: Sender<MailPayload>) -> Self {
        Self { sender }
    }

    pub fn sender(&self) -> &Sender<MailPayload> {
        &self.sender
    }
}

pub struct DaoCtx {
    db: Db,
}

impl DaoCtx {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &Db {
        &self.db
    }
}
