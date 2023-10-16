use hb_config::DbScyllaConfig;
use hb_db::Repository;
use repository::ScyllaRepository;
use scylla::SessionBuilder;

pub mod model;
mod repository;

pub async fn new(config: &DbScyllaConfig) -> impl Repository {
    let uri = format!("{}:{}", config.host, config.port);
    let session = SessionBuilder::new().known_node(uri).build().await.unwrap();
    ScyllaRepository { session }
}
