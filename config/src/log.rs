use serde::Deserialize;

#[derive(Deserialize)]
pub struct LogConfig {
    display_level: bool,
    level_filter: String,
}

impl LogConfig {
    pub fn display_level(&self) -> &bool {
        &self.display_level
    }

    pub fn level_filter(&self) -> &str {
        &self.level_filter
    }
}
