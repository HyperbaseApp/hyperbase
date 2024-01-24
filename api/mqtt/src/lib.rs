use anyhow::Result;
use context::ApiMqttCtx;
use model::payload::Payload;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use service::record::record_service;

pub mod context;
mod model;
mod service;

pub struct ApiMqttClient {
    host: String,
    port: u16,
    topic: String,
    channel_capacity: usize,
    context: ApiMqttCtx,
}

impl ApiMqttClient {
    pub fn new(
        host: &str,
        port: &u16,
        topic: &str,
        channel_capacity: &usize,
        ctx: ApiMqttCtx,
    ) -> Self {
        hb_log::info(Some("⚡"), "ApiMqttClient: Initializing component");

        Self {
            host: host.to_owned(),
            port: *port,
            topic: topic.to_owned(),
            channel_capacity: *channel_capacity,
            context: ctx,
        }
    }

    pub async fn run(&self) -> Result<()> {
        hb_log::info(Some("💫"), "ApiMqttClient: Running component");

        let mqtt_opts = MqttOptions::new("rumqtt-async", &self.host, self.port);

        let (client, mut eventloop) = AsyncClient::new(mqtt_opts, self.channel_capacity);
        client
            .subscribe(&self.topic, QoS::AtMostOnce)
            .await
            .unwrap();

        loop {
            if let Ok(event) = eventloop.poll().await {
                if let Event::Incoming(packet) = event {
                    if let Packet::Publish(publish) = packet {
                        match serde_json::from_slice::<Payload>(&publish.payload) {
                            Ok(payload) => record_service(&self.context, &payload).await,
                            Err(err) => hb_log::error(None, err),
                        }
                    }
                }
            }
        }
    }
}
