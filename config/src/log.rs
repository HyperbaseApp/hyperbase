use serde::Deserialize;

#[derive(Deserialize)]
pub struct LogConfig {
    display_level: bool,
    level_filter: String,
    log_ttl: u32,
}

impl LogConfig {
    pub fn display_level(&self) -> &bool {
        &self.display_level
    }

    pub fn level_filter(&self) -> &str {
        &self.level_filter
    }

    pub fn log_ttl(&self) -> &u32 {
        &self.log_ttl
    }
}
