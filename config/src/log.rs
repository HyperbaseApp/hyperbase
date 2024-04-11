use serde::Deserialize;

#[derive(Deserialize)]
pub struct LogConfig {
    display_level: bool,
    level_filter: String,
    db_ttl: u32,
}

impl LogConfig {
    pub fn display_level(&self) -> &bool {
        &self.display_level
    }

    pub fn level_filter(&self) -> &str {
        &self.level_filter
    }

    pub fn db_ttl(&self) -> &u32 {
        &self.db_ttl
    }
}
