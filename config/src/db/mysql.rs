use serde::Deserialize;

#[derive(Deserialize)]
pub struct DbMysqlConfig {
    user: String,
    password: String,
    host: String,
    port: String,
    db_name: String,
    max_connections: u32,
}

impl DbMysqlConfig {
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

    pub fn db_name(&self) -> &str {
        &self.db_name
    }

    pub fn max_connections(&self) -> &u32 {
        &self.max_connections
    }
}
