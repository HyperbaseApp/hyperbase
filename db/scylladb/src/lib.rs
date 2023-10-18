use db::ScyllaDb;
use hb_config::DbScyllaConfig;

mod db;
pub mod model;
mod prepared_statement;

pub async fn new(config: &DbScyllaConfig) -> ScyllaDb {
    ScyllaDb::new(config).await
}
