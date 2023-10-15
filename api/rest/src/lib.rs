use actix_web::{App, HttpServer};
use hb_config::ApiRestConfig;
use service::collection::collection_api;

mod service;

pub async fn run(config: &ApiRestConfig) {
    let addrs = format!("{}:{}", config.host, config.port);
    HttpServer::new(|| App::new().configure(collection_api))
        .bind(addrs)
        .unwrap()
        .run()
        .await
        .unwrap();
}
