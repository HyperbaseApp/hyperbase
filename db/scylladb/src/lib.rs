use hb_config::DbScyllaConfig;
use hb_db::DbRepository;
use repository::ScyllaRepository;
use scylla::SessionBuilder;

mod models;
mod repository;

pub async fn new(config: &DbScyllaConfig) -> impl DbRepository {
    let uri = format!("{}:{}", config.host, config.port);
    let session = SessionBuilder::new().known_node(uri).build().await.unwrap();
    ScyllaRepository { session }
}
