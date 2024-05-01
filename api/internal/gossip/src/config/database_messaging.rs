pub struct DatabaseMessagingConfig {
    push: bool,
    pull: bool,
    period: u64,           // in ms
    period_deviation: u64, // in ms
    actions_size: usize,
}

impl DatabaseMessagingConfig {
    pub fn push(&self) -> &bool {
        &self.push
    }

    pub fn pull(&self) -> &bool {
        &self.pull
    }

    pub fn period(&self) -> &u64 {
        &self.period
    }

    pub fn period_deviation(&self) -> &u64 {
        &self.period_deviation
    }

    pub fn actions_size(&self) -> &usize {
        &self.actions_size
    }
}

impl Default for DatabaseMessagingConfig {
    fn default() -> Self {
        Self {
            push: true,
            pull: true,
            period: 60000,
            period_deviation: 10000,
            actions_size: Default::default(),
        }
    }
}
