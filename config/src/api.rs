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
    port: String,
}

impl ApiRestConfig {
    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> &str {
        &self.port
    }
}

#[derive(Deserialize)]
pub struct ApiMqttConfig {
    host: String,
    port: String,
}

impl ApiMqttConfig {
    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> &str {
        &self.port
    }
}
