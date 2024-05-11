use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use context::ApiMqttCtx;
use model::payload::Payload;
use rumqttc::v5::{
    mqttbytes::{v5::Packet, QoS},
    AsyncClient, Event, EventLoop, MqttOptions,
};
use service::record::record_service;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub mod context;
mod model;
mod service;
mod util;

pub struct ApiMqttClient {
    client: AsyncClient,
    eventloop: EventLoop,
    topic: String,
    timeout: Duration,
    context: Arc<ApiMqttCtx>,
}

impl ApiMqttClient {
    pub fn new(
        host: &str,
        port: &u16,
        topic: &str,
        username: &str,
        password: &str,
        channel_capacity: &usize,
        timeout: &Duration,
        ctx: ApiMqttCtx,
    ) -> Self {
        hb_log::info(Some("⚡"), "[ApiMqttClient] Initializing component");

        let mut mqtt_opts = MqttOptions::new(format!("hyperbase-{}", Uuid::now_v7()), host, *port);
        mqtt_opts.set_credentials(username, password);

        let (client, eventloop) = AsyncClient::new(mqtt_opts, *channel_capacity);

        Self {
            client,
            eventloop,
            topic: topic.to_owned(),
            timeout: *timeout,
            context: Arc::new(ctx),
        }
    }

    pub fn run_none() -> JoinHandle<()> {
        hb_log::info(Some("⏩"), "[ApiMqttClient] Skipping component");

        tokio::spawn((|| async {})())
    }

    pub fn run(mut self, cancel_token: CancellationToken) -> JoinHandle<()> {
        hb_log::info(Some("💫"), "[ApiMqttClient] Running component");

        tokio::spawn((|| async move {
            self.client
                .subscribe(self.topic, QoS::ExactlyOnce)
                .await
                .unwrap();

            let mut now = Instant::now();

            loop {
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        break;
                    }
                    _ = tokio::signal::ctrl_c() => {
                        break;
                    }
                    poll_result = self.eventloop.poll() => {
                        if let Ok(event) = poll_result {
                            now = Instant::now();
                            if let Event::Incoming(packet) = &event {
                                if let Packet::Publish(publish) = packet {
                                    match serde_json::from_slice::<Payload>(&publish.payload) {
                                        Ok(payload) => {
                                            let ctx = self.context.clone();
                                            let payload = payload.clone();
                                            tokio::spawn((|| async move {
                                                record_service(&ctx, &payload).await;
                                            })());
                                        }
                                        Err(err) => hb_log::error(None, &format!("[ApiMqttClient] {err}")),
                                    };
                                }
                            }
                            continue;
                        }
                    }
                }
                if Instant::now().duration_since(now) > self.timeout {
                    hb_log::panic(
                        None,
                        &format!(
                            "[ApiMqttClient] Failed to connect to MQTT broker {:?}",
                            self.eventloop.options.broker_address()
                        ),
                    );
                }
            }

            hb_log::info(None, "[ApiMqttClient] Shutting down component");
            let _ = self.client.disconnect().await;
        })())
    }
}
