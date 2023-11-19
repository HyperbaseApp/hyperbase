use std::sync::mpsc::Sender;

use hb_dao::Db;
use hb_hash_argon2::argon2::Argon2Hash;
use hb_mailer::MailPayload;
use hb_token_jwt::token::JwtToken;

pub struct Context {
    pub hash: HashCtx,
    pub token: TokenCtx,
    pub mailer: MailerCtx,
    pub dao: DaoCtx,
    pub verification_code_ttl: i64,
}

pub struct HashCtx {
    pub argon2: Argon2Hash,
}

pub struct TokenCtx {
    pub jwt: JwtToken,
}

pub struct MailerCtx {
    pub sender: Sender<MailPayload>,
}

pub struct DaoCtx {
    pub db: Db,
}
