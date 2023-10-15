use std::fs::File;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub db: DbConfig,
    pub api: ApiConfig,
}

#[derive(Deserialize)]
pub struct DbConfig {
    pub scylla: DbScyllaConfig,
}

#[derive(Deserialize)]
pub struct DbScyllaConfig {
    pub host: String,
    pub port: String,
}

#[derive(Deserialize)]
pub struct ApiConfig {
    pub rest: ApiRestConfig,
}

#[derive(serde::Deserialize)]
pub struct ApiRestConfig {
    pub host: String,
    pub port: String,
}

pub fn new(path: &str) -> Config {
    let file = File::open(path)
        .expect("Failed to parse the configuration file in the CONFIG_PATH environment variable");
    serde_yaml::from_reader::<_, Config>(file).unwrap()
}
