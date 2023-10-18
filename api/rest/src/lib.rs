use actix_web::{web, App, HttpServer};
use hb_config::ApiRestConfig;
use v1::v1_api;

mod v1;

pub async fn run(config: &ApiRestConfig) {
    let addrs = format!("{}:{}", config.host(), config.port());
    HttpServer::new(|| App::new().service(web::scope("/api/rest/v1").configure(v1_api)))
        .bind(addrs)
        .unwrap()
        .run()
        .await
        .unwrap();
}
