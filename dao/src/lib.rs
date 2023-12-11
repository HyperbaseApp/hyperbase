use hb_db_scylladb::db::ScyllaDb;

pub mod admin;
pub mod admin_password_reset;
pub mod collection;
pub mod dto;
pub mod project;
pub mod record;
pub mod register;
pub mod token;
mod util;

pub enum Db {
    ScyllaDb(ScyllaDb),
}
