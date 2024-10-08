use hb_db_mysql::db::MysqlDb;
use hb_db_postgresql::db::PostgresDb;
use hb_db_scylladb::db::ScyllaDb;
use hb_db_sqlite::db::SqliteDb;

pub mod admin;
pub mod admin_password_reset;
pub mod bucket;
pub mod bucket_rule;
pub mod collection;
pub mod collection_rule;
pub mod file;
pub mod log;
pub mod project;
pub mod record;
pub mod registration;
pub mod token;
mod util;
pub mod value;

pub enum Db {
    ScyllaDb(ScyllaDb),
    PostgresqlDb(PostgresDb),
    MysqlDb(MysqlDb),
    SqliteDb(SqliteDb),
}
