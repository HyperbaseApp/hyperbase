#[derive(Clone, Copy)]
pub struct DatabaseMessagingConfig {
    period: u64,           // in ms
    period_deviation: u64, // in ms
    actions_size: i32,
}

impl DatabaseMessagingConfig {
    pub fn period(&self) -> &u64 {
        &self.period
    }

    pub fn period_deviation(&self) -> &u64 {
        &self.period_deviation
    }

    pub fn actions_size(&self) -> &i32 {
        &self.actions_size
    }
}

impl Default for DatabaseMessagingConfig {
    fn default() -> Self {
        Self {
            period: 5000,
            period_deviation: 5000,
            actions_size: 30,
        }
    }
}
