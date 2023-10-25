use std::sync::mpsc::Sender;

use hb_db_scylladb::db::ScyllaDb;
use hb_hash_argon2::argon2::Argon2Hash;
use hb_mailer::MailPayload;
use hb_token_jwt::context::JwtToken;

pub struct ApiRestContext {
    pub hash: HashCtx,
    pub token: TokenCtx,
    pub mailer: MailerCtx,
    pub db: DbCtx,
    pub verify_code_ttl: i64,
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

pub struct DbCtx {
    pub scylladb: ScyllaDb,
}
