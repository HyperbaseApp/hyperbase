use std::fs::File;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    hash: HashConfig,
    mailer: MailerConfig,
    db: DbConfig,
    api: ApiConfig,
}

impl Config {
    pub fn hash(&self) -> &HashConfig {
        &self.hash
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
    temp_ttl: i64,
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

    pub fn temp_ttl(&self) -> &i64 {
        &self.temp_ttl
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

#[derive(serde::Deserialize)]
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

pub fn new(path: &str) -> Config {
    let file = File::open(path)
        .expect("Failed to parse the configuration file in the CONFIG_PATH environment variable");
    serde_yaml::from_reader::<_, Config>(file).unwrap()
}
