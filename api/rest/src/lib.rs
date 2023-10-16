use actix_web::{App, HttpServer};
use hb_config::ApiRestConfig;
use service::{admin::admin_api, collection::collection_api, project::project_api};

mod model;
mod service;

pub async fn run(config: &ApiRestConfig) {
    let addrs = format!("{}:{}", config.host, config.port);
    HttpServer::new(|| {
        App::new()
            .configure(admin_api)
            .configure(project_api)
            .configure(collection_api)
    })
    .bind(addrs)
    .unwrap()
    .run()
    .await
    .unwrap();
}
