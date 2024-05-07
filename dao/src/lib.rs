use anyhow::Result;
use hb_db_mysql::db::MysqlDb;
use hb_db_postgresql::db::PostgresDb;
use hb_db_scylladb::db::ScyllaDb;
use hb_db_sqlite::db::SqliteDb;

pub mod admin;
pub mod admin_password_reset;
pub mod bucket;
pub mod bucket_rule;
pub mod change;
pub mod collection;
pub mod collection_rule;
pub mod file;
mod init;
pub mod local_info;
pub mod log;
pub mod project;
pub mod record;
pub mod registration;
pub mod remote_sync;
pub mod token;
mod util;
pub mod value;

pub enum Db {
    ScyllaDb(ScyllaDb),
    PostgresqlDb(PostgresDb),
    MysqlDb(MysqlDb),
    SqliteDb(SqliteDb),
}

impl Db {
    pub async fn init(&self) -> Result<()> {
        hb_log::info(Some("⚡"), "[DAO] Initializing component");

        self.update_change().await?;
        Ok(())
    }
}
