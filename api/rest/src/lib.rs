use actix_cors::Cors;
use actix_web::{
    middleware::{ErrorHandlers, Logger},
    web, App, HttpServer,
};
use anyhow::Result;
use configure::configure;
use context::ApiRestCtx;
use error_handler::default_error_handler;
use hb_config::app::AppConfigMode;
use logger::logger_format;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

mod configure;
pub mod context;
mod error_handler;
mod logger;
mod model;
mod service;

pub struct ApiRestServer {
    app_mode: AppConfigMode,
    address: String,
    allowed_origin: Option<String>,
    context: web::Data<ApiRestCtx>,
}

impl ApiRestServer {
    pub fn new(
        app_mode: &AppConfigMode,
        host: &str,
        port: &u16,
        allowed_origin: &Option<String>,
        ctx: ApiRestCtx,
    ) -> Self {
        hb_log::info(Some("⚡"), "ApiRestServer: Initializing component");

        let address = format!("{}:{}", host, port);
        let context = web::Data::new(ctx);

        Self {
            app_mode: *app_mode,
            address,
            allowed_origin: allowed_origin.to_owned(),
            context,
        }
    }

    pub fn run(self, cancel_token: CancellationToken) -> JoinHandle<Result<()>> {
        hb_log::info(Some("💫"), "ApiRestServer: Running component");

        tokio::spawn((|| async move {
            let server = HttpServer::new(move || {
                App::new()
                    .wrap((|| -> Cors {
                        if matches!(self.app_mode, AppConfigMode::Production) {
                            let cors = Cors::default().allow_any_header().allow_any_method();
                            if let Some(origin) = &self.allowed_origin {
                                cors.allowed_origin(origin)
                            } else {
                                cors
                            }
                        } else {
                            Cors::permissive()
                        }
                    })())
                    .wrap(Logger::new(logger_format()))
                    .wrap(ErrorHandlers::new().default_handler(default_error_handler))
                    .app_data(self.context.clone())
                    .configure(configure)
            })
            .bind(self.address)
            .unwrap()
            .run();

            let server_handle = server.handle();

            tokio::select! {
                _ = cancel_token.cancelled() => {}
                _ = server => {}
            }

            hb_log::info(None, "ApiRestServer: Shutting down component");
            server_handle.stop(true).await;

            Ok(())
        })())
    }
}
