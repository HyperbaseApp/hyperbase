use serde::Deserialize;

#[derive(Deserialize)]
pub struct DbScyllaConfig {
    user: String,
    password: String,
    host: String,
    port: String,
    replication_factor: i64,
    prepared_statement_cache_size: usize,
}

impl DbScyllaConfig {
    pub fn user(&self) -> &str {
        &self.user
    }

    pub fn password(&self) -> &str {
        &self.password
    }

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
}
