use std::{fs::File, path::Path};

use api::ApiConfig;
use app::AppConfig;
use auth::AuthConfig;
use bucket::BucketConfig;
use db::DbConfig;
use hash::HashConfig;
use log::LogConfig;
use mailer::MailerConfig;
use serde::Deserialize;
use token::TokenConfig;

pub mod api;
pub mod app;
pub mod auth;
pub mod bucket;
pub mod db;
pub mod hash;
pub mod log;
pub mod mailer;
pub mod token;

#[derive(Deserialize)]
pub struct Config {
    app: AppConfig,
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
    pub fn app(&self) -> &AppConfig {
        &self.app
    }

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

pub fn from_path(path: &Path) -> Config {
    let file = File::open(path).expect("");
    serde_yaml::from_reader::<_, Config>(file).unwrap()
}
