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
use service::Service;
use tokio::{sync::mpsc, task::JoinHandle};
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
    service: Service,
    payload_sender: mpsc::UnboundedSender<Payload>,
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

        let (service, payload_sender) = Service::new(Arc::new(ctx));

        Self {
            client,
            eventloop,
            topic: topic.to_owned(),
            timeout: *timeout,
            service,
            payload_sender,
        }
    }

    pub fn run_none() -> JoinHandle<()> {
        hb_log::info(Some("⏩"), "[ApiMqttClient] Skipping component");

        tokio::spawn((|| async {})())
    }

    pub fn run(self, cancel_token: CancellationToken) -> JoinHandle<()> {
        hb_log::info(Some("💫"), "[ApiMqttClient] Running component");

        tokio::spawn((|| async move {
            let service = self.service.run();

            self.client
                .subscribe(self.topic, QoS::ExactlyOnce)
                .await
                .unwrap();

            tokio::select! {
                _ = cancel_token.cancelled() => {}
                _ = tokio::signal::ctrl_c() => {}
                s = service => {
                    if let Err(err) = s {
                        hb_log::panic(None, &format!("[ApiMqttClient] Receiver service error: {err}"));
                    }
                }
                _ = Self::poll(self.eventloop, self.timeout, self.payload_sender) => {}
            }

            hb_log::info(None, "[ApiMqttClient] Shutting down component");
            let _ = self.client.disconnect().await;
        })())
    }

    async fn poll(
        mut eventloop: EventLoop,
        timeout: Duration,
        payload_sender: mpsc::UnboundedSender<Payload>,
    ) {
        let mut now = Instant::now();

        loop {
            if let Ok(event) = eventloop.poll().await {
                now = Instant::now();
                if let Event::Incoming(packet) = &event {
                    if let Packet::Publish(publish) = packet {
                        match serde_json::from_slice::<Payload>(&publish.payload) {
                            Ok(payload) => {
                                if let Err(err) = payload_sender.send(payload) {
                                    hb_log::error(
                                        None,
                                        &format!("[ApiMqttClient] Send payload error: {err}"),
                                    );
                                }
                            }
                            Err(err) => hb_log::error(
                                None,
                                &format!("[ApiMqttClient] Payload deserialize error: {err}"),
                            ),
                        };
                    }
                }
                continue;
            }
            if Instant::now().duration_since(now) > timeout {
                hb_log::panic(None, "[ApiMqttClient] Failed to connect to MQTT broker");
            }
        }
    }
}
