use std::fs::File;

use api::ApiConfig;
use auth::AuthConfig;
use bucket::BucketConfig;
use db::DbConfig;
use hash::HashConfig;
use log::LogConfig;
use mailer::MailerConfig;
use serde::Deserialize;
use token::TokenConfig;

pub mod api;
pub mod auth;
pub mod bucket;
pub mod db;
pub mod hash;
pub mod log;
pub mod mailer;
pub mod token;

#[derive(Deserialize)]
pub struct Config {
    log: LogConfig,
    hash: HashConfig,
    token: TokenConfig,
    mailer: MailerConfig,
    db: DbConfig,
    bucket: BucketConfig,
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

    pub fn bucket(&self) -> &BucketConfig {
        &self.bucket
    }

    pub fn api(&self) -> &ApiConfig {
        &self.api
    }

    pub fn auth(&self) -> &AuthConfig {
        &self.auth
    }
}

pub fn new(path: &str) -> Config {
    let file = File::open(path).expect("");
    serde_yaml::from_reader::<_, Config>(file).unwrap()
}
