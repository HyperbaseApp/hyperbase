use actix_web::{web, App, HttpServer};
use context::ApiRestContext as Context;
use v1::v1_api;

pub mod context;
mod v1;

pub struct ApiRestServer {
    address: String,
    context: web::Data<Context>,
}

impl ApiRestServer {
    pub fn new(host: &str, port: &str, ctx: Context) -> Self {
        let address = format!("{}:{}", host, port);
        let context = web::Data::new(ctx);

        Self { address, context }
    }

    pub async fn run(self) {
        HttpServer::new(move || {
            App::new()
                .app_data(self.context.clone())
                .service(web::scope("/api/rest/v1").configure(v1_api))
        })
        .bind(self.address)
        .unwrap()
        .run()
        .await
        .unwrap();
    }
}
