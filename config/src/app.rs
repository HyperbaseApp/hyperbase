use serde::Deserialize;

#[derive(Deserialize)]
pub struct AppConfig {
    mode: AppConfigMode,
}

impl AppConfig {
    pub fn mode(&self) -> &AppConfigMode {
        &self.mode
    }
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum AppConfigMode {
    Development,
    Production,
}
