use std::sync::Arc;

use anyhow::Result;
use context::ApiMqttCtx;
use handshake::{handshake_v3, handshake_v5};
use ntex::server::Server;
use ntex_mqtt::{v3, v5, MqttServer};
use publish::{v3_publish, v5_publish};

pub mod context;
mod error_handler;
mod handshake;
mod model;
mod publish;
mod service;
mod session;

pub struct ApiMqttServer {
    address: String,
    context: Arc<ApiMqttCtx>,
}

impl ApiMqttServer {
    pub fn new(host: &str, port: &str, ctx: ApiMqttCtx) -> Self {
        hb_log::info(Some("⚡"), "ApiMqttServer: Initializing component");

        let address = format!("{}:{}", host, port);
        let context = Arc::new(ctx);

        Self { address, context }
    }

    pub async fn run(self) -> Result<()> {
        hb_log::info(Some("💫"), "ApiMqttServer: Running component");

        Ok(Server::build()
            .bind("mqtt", &self.address, move |_| {
                let v3_ctx = self.context.clone();
                let v5_ctx = self.context.clone();

                MqttServer::new()
                    .v3(v3::MqttServer::new(move |handshake| {
                        handshake_v3(v3_ctx.clone(), handshake)
                    })
                    .publish(v3_publish(&self.context)))
                    .v5(v5::MqttServer::new(move |handshake| {
                        handshake_v5(v5_ctx.clone(), handshake)
                    })
                    .publish(v5_publish(&self.context)))
            })?
            .run()
            .await?)
    }
}
