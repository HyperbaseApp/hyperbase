use chrono::Duration;
use duration_str::deserialize_duration_chrono;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ApiConfig {
    rest: ApiRestConfig,
    mqtt: ApiMqttConfig,
}

impl ApiConfig {
    pub fn rest(&self) -> &ApiRestConfig {
        &self.rest
    }

    pub fn mqtt(&self) -> &ApiMqttConfig {
        &self.mqtt
    }
}

#[derive(Deserialize)]
pub struct ApiRestConfig {
    host: String,
    port: u16,
    allowed_origin: Option<String>,
}

impl ApiRestConfig {
    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> &u16 {
        &self.port
    }

    pub fn allowed_origin(&self) -> &Option<String> {
        &self.allowed_origin
    }
}

#[derive(Deserialize)]
pub struct ApiMqttConfig {
    host: String,
    port: u16,
    topic: String,
    channel_capacity: usize,
    #[serde(deserialize_with = "deserialize_duration_chrono")]
    timeout: Duration,
}

impl ApiMqttConfig {
    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> &u16 {
        &self.port
    }

    pub fn topic(&self) -> &str {
        &self.topic
    }

    pub fn channel_capacity(&self) -> &usize {
        &self.channel_capacity
    }

    pub fn timeout(&self) -> &Duration {
        &self.timeout
    }
}
