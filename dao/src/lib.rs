use hb_db_scylladb::db::ScyllaDb;

pub mod admin;
pub mod admin_password_reset;
pub mod collection;
pub mod project;
pub mod register;
pub mod token;
mod util;

pub enum Db<'a> {
    ScyllaDb(&'a ScyllaDb),
}
