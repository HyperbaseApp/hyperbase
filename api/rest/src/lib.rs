use actix_web::{web, App, HttpServer};
use context::Context;
use hb_config::ApiRestConfig;
use v1::v1_api;

pub mod context;
mod v1;

pub async fn run(config: &ApiRestConfig, ctx: Context) {
    let addrs = format!("{}:{}", config.host(), config.port());

    let data = web::Data::new(ctx);

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(web::scope("/api/rest/v1").configure(v1_api))
    })
    .bind(addrs)
    .unwrap()
    .run()
    .await
    .unwrap();
}
