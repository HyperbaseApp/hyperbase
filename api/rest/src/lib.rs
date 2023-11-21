use actix_web::{
    middleware::{ErrorHandlers, Logger},
    web, App, HttpServer,
};
use anyhow::Result;
use context::Context;
use error_handler::default_error_handler;
use v1::v1_api;

pub mod context;
mod error_handler;
mod v1;

pub struct ApiRestServer {
    address: String,
    context: web::Data<Context>,
}

impl ApiRestServer {
    pub fn new(host: &str, port: &str, ctx: Context) -> Self {
        hb_log::info(Some("⚡"), "Creating component: ApiRestServer");

        let address = format!("{}:{}", host, port);
        let context = web::Data::new(ctx);

        Self { address, context }
    }

    pub async fn run(self) -> Result<()> {
        hb_log::info(Some("💫"), "Running component: ApiRestServer");

        Ok(HttpServer::new(move || {
            App::new()
                .wrap(Logger::default())
                .wrap(ErrorHandlers::new().default_handler(default_error_handler))
                .app_data(self.context.clone())
                .service(web::scope("/api/rest/v1").configure(v1_api))
        })
        .bind(self.address)
        .unwrap()
        .run()
        .await?)
    }
}
