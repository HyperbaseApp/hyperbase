use anyhow::Result;
use rumqttc::{AsyncClient, MqttOptions, QoS};

pub struct ApiMqttClient {
    host: String,
    port: u16,
    topic: String,
    channel_capacity: usize,
}

impl ApiMqttClient {
    pub fn new(host: &str, port: &u16, topic: &str, channel_capacity: &usize) -> Self {
        hb_log::info(Some("⚡"), "ApiMqttClient: Initializing component");

        Self {
            host: host.to_owned(),
            port: *port,
            topic: topic.to_owned(),
            channel_capacity: *channel_capacity,
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

        while let Ok(notification) = eventloop.poll().await {
            println!("Received = {:?}", notification);
        }

        Ok(())
    }
}
