use std::fs::File;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    log: LogConfig,
    hash: HashConfig,
    token: TokenConfig,
    mailer: MailerConfig,
    db: DbConfig,
    api: ApiConfig,
    auth: AuthConfig,
}

impl Config {
    pub fn log(&self) -> &LogConfig {
        &self.log
    }

    pub fn hash(&self) -> &HashConfig {
        &self.hash
    }

    pub fn token(&self) -> &TokenConfig {
        &self.token
    }

    pub fn mailer(&self) -> &MailerConfig {
        &self.mailer
    }

    pub fn db(&self) -> &DbConfig {
        &self.db
    }

    pub fn api(&self) -> &ApiConfig {
        &self.api
    }

    pub fn auth(&self) -> &AuthConfig {
        &self.auth
    }
}

#[derive(Deserialize)]
pub struct LogConfig {
    display_level: bool,
    level_filter: String,
}

impl LogConfig {
    pub fn display_level(&self) -> &bool {
        &self.display_level
    }

    pub fn level_filter(&self) -> &str {
        &self.level_filter
    }
}

#[derive(Deserialize)]
pub struct HashConfig {
    argon2: Argon2HashConfig,
}

impl HashConfig {
    pub fn argon2(&self) -> &Argon2HashConfig {
        &self.argon2
    }
}

#[derive(Deserialize)]
pub struct Argon2HashConfig {
    algorithm: String,
    version: String,
    salt: String,
}

impl Argon2HashConfig {
    pub fn algorithm(&self) -> &str {
        &self.algorithm
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn salt(&self) -> &str {
        &self.salt
    }
}

#[derive(Deserialize)]
pub struct TokenConfig {
    jwt: JwtTokenConfig,
}

impl TokenConfig {
    pub fn jwt(&self) -> &JwtTokenConfig {
        &self.jwt
    }
}

#[derive(Deserialize)]
pub struct JwtTokenConfig {
    secret: String,
    expiry_duration: u64,
}

impl JwtTokenConfig {
    pub fn secret(&self) -> &str {
        &self.secret
    }

    pub fn expiry_duration(&self) -> &u64 {
        &self.expiry_duration
    }
}

#[derive(Deserialize)]
pub struct MailerConfig {
    smtp_host: String,
    smtp_username: String,
    smtp_password: String,
    sender_name: String,
    sender_email: String,
}

impl MailerConfig {
    pub fn smtp_host(&self) -> &str {
        &self.smtp_host
    }

    pub fn smtp_username(&self) -> &str {
        &self.smtp_username
    }

    pub fn smtp_password(&self) -> &str {
        &self.smtp_password
    }

    pub fn sender_name(&self) -> &str {
        &self.sender_name
    }

    pub fn sender_email(&self) -> &str {
        &self.sender_email
    }
}

#[derive(Deserialize)]
pub struct DbConfig {
    scylla: DbScyllaConfig,
}

impl DbConfig {
    pub fn scylla(&self) -> &DbScyllaConfig {
        &self.scylla
    }
}

#[derive(Deserialize)]
pub struct DbScyllaConfig {
    host: String,
    port: String,
    replication_factor: i64,
    prepared_statement_cache_size: usize,
    table_properties: DbScyllaConfigTableProperties,
}

impl DbScyllaConfig {
    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> &str {
        &self.port
    }

    pub fn replication_factor(&self) -> &i64 {
        &self.replication_factor
    }

    pub fn prepared_statement_cache_size(&self) -> &usize {
        &self.prepared_statement_cache_size
    }

    pub fn table_properties(&self) -> &DbScyllaConfigTableProperties {
        &self.table_properties
    }
}

#[derive(Deserialize)]
pub struct DbScyllaConfigTableProperties {
    registration_ttl: i64,
    reset_password_ttl: i64,
}

impl DbScyllaConfigTableProperties {
    pub fn registration_ttl(&self) -> &i64 {
        &self.registration_ttl
    }

    pub fn reset_password_ttl(&self) -> &i64 {
        &self.reset_password_ttl
    }
}

#[derive(Deserialize)]
pub struct ApiConfig {
    rest: ApiRestConfig,
}

impl ApiConfig {
    pub fn rest(&self) -> &ApiRestConfig {
        &self.rest
    }
}

#[derive(Deserialize)]
pub struct ApiRestConfig {
    host: String,
    port: String,
}

impl ApiRestConfig {
    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> &str {
        &self.port
    }
}

#[derive(Deserialize)]
pub struct AuthConfig {
    access_token_length: usize,
}

impl AuthConfig {
    pub fn access_token_length(&self) -> &usize {
        &self.access_token_length
    }
}

pub fn new(path: &str) -> Config {
    let file = File::open(path).expect("");
    serde_yaml::from_reader::<_, Config>(file).unwrap()
}
