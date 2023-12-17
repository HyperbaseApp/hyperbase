use serde::Deserialize;

#[derive(Deserialize)]
pub struct ApiConfig {
    rest: ApiRestConfig,
}

impl ApiConfig {
    pub fn rest(&self) -> &ApiRestConfig {
        &self.rest
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
