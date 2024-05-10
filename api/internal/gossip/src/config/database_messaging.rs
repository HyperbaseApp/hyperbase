#[derive(Clone, Copy)]
pub struct DatabaseMessagingConfig {
    period: u64,           // in ms
    period_deviation: u64, // in ms
    actions_size: i32,
    max_broadcast: u32,
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

    pub fn max_broadcast(&self) -> &u32 {
        &self.max_broadcast
    }
}

impl Default for DatabaseMessagingConfig {
    fn default() -> Self {
        Self {
            period: 60000,
            period_deviation: 10000,
            actions_size: 30,
            max_broadcast: 3,
        }
    }
}
