use actix_web::{
    middleware::{ErrorHandlers, Logger},
    web, App, HttpServer,
};
use anyhow::Result;
use config::config;
use context::ApiRestCtx;
use error_handler::default_error_handler;
use logger::logger_format;

mod config;
pub mod context;
mod error_handler;
mod logger;
mod model;
mod service;

pub struct ApiRestServer {
    address: String,
    context: web::Data<ApiRestCtx>,
}

impl ApiRestServer {
    pub fn new(host: &str, port: &str, ctx: ApiRestCtx) -> Self {
        hb_log::info(Some("⚡"), "ApiRestServer: Initializing component");

        let address = format!("{}:{}", host, port);
        let context = web::Data::new(ctx);

        Self { address, context }
    }

    pub async fn run(self) -> Result<()> {
        hb_log::info(Some("💫"), "ApiRestServer: Running component");

        Ok(HttpServer::new(move || {
            App::new()
                .wrap(Logger::new(logger_format()))
                .wrap(ErrorHandlers::new().default_handler(default_error_handler))
                .app_data(self.context.clone())
                .configure(config)
        })
        .bind(self.address)
        .unwrap()
        .run()
        .await?)
    }
}
