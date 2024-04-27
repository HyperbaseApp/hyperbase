use std::time::Duration;

use duration_str::deserialize_duration;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ApiConfig {
    rest: ApiRestConfig,
    websocket: ApiWebSocketConfig,
    mqtt: Option<ApiMqttConfig>,
}

impl ApiConfig {
    pub fn rest(&self) -> &ApiRestConfig {
        &self.rest
    }

    pub fn websocket(&self) -> &ApiWebSocketConfig {
        &self.websocket
    }

    pub fn mqtt(&self) -> &Option<ApiMqttConfig> {
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
pub struct ApiWebSocketConfig {
    #[serde(deserialize_with = "deserialize_duration")]
    heartbeat_interval: Duration,
    #[serde(deserialize_with = "deserialize_duration")]
    client_timeout: Duration,
}

impl ApiWebSocketConfig {
    pub fn heartbeat_interval(&self) -> &Duration {
        &self.heartbeat_interval
    }

    pub fn client_timeout(&self) -> &Duration {
        &self.client_timeout
    }
}

#[derive(Deserialize)]
pub struct ApiMqttConfig {
    host: String,
    port: u16,
    topic: String,
    username: String,
    password: String,
    channel_capacity: usize,
    #[serde(deserialize_with = "deserialize_duration")]
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

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn channel_capacity(&self) -> &usize {
        &self.channel_capacity
    }

    pub fn timeout(&self) -> &Duration {
        &self.timeout
    }
}
