use std::time::Duration;

use duration_str::deserialize_duration;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ApiConfig {
    internal: Option<ApiInternalConfig>,
    rest: ApiRestConfig,
    websocket: ApiWebSocketConfig,
    mqtt: Option<ApiMqttConfig>,
}

impl ApiConfig {
    pub fn internal(&self) -> &Option<ApiInternalConfig> {
        &self.internal
    }

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
pub struct ApiInternalConfig {
    gossip: Option<ApiGossipConfig>,
}

impl ApiInternalConfig {
    pub fn gossip(&self) -> &Option<ApiGossipConfig> {
        &self.gossip
    }
}

#[derive(Deserialize)]
pub struct ApiGossipConfig {
    host: String,
    port: u16,
    peers: Option<Vec<String>>,
    view_size: usize,
    actions_size: i32,
}

impl ApiGossipConfig {
    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> &u16 {
        &self.port
    }

    pub fn peers(&self) -> &Option<Vec<String>> {
        &self.peers
    }

    pub fn view_size(&self) -> &usize {
        &self.view_size
    }

    pub fn actions_size(&self) -> &i32 {
        &self.actions_size
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
