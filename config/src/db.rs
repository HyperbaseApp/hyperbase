use serde::Deserialize;

use self::{
    mysql::DbMysqlConfig, postgres::DbPostgresConfig, scylla::DbScyllaConfig,
    sqlite::DbSqliteConfig,
};

pub mod mysql;
pub mod postgres;
pub mod scylla;
pub mod sqlite;

#[derive(Deserialize)]
pub struct DbConfig {
    scylla: Option<DbScyllaConfig>,
    postgres: Option<DbPostgresConfig>,
    mysql: Option<DbMysqlConfig>,
    sqlite: Option<DbSqliteConfig>,
}

impl DbConfig {
    pub fn scylla(&self) -> &Option<DbScyllaConfig> {
        &self.scylla
    }

    pub fn postgres(&self) -> &Option<DbPostgresConfig> {
        &self.postgres
    }

    pub fn mysql(&self) -> &Option<DbMysqlConfig> {
        &self.mysql
    }

    pub fn sqlite(&self) -> &Option<DbSqliteConfig> {
        &self.sqlite
    }
}
