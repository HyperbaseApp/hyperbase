use serde::Deserialize;

#[derive(Deserialize)]
pub struct DbSqliteConfig {
    path: String,
    max_connections: u32,
}

impl DbSqliteConfig {
    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn max_connections(&self) -> &u32 {
        &self.max_connections
    }
}
