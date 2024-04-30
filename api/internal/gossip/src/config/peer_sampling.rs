#[derive(Clone, Copy)]
pub struct PeerSamplingConfig {
    push: bool,
    pull: bool,
    period: u64,           // in ms
    period_deviation: u64, // in ms
    view_size: usize,
    healing_factor: usize,
    swapping_factor: usize,
}

impl PeerSamplingConfig {
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

    pub fn view_size(&self) -> &usize {
        &self.view_size
    }

    pub fn healing_factor(&self) -> &usize {
        &self.healing_factor
    }

    pub fn swapping_factor(&self) -> &usize {
        &self.swapping_factor
    }
}

impl Default for PeerSamplingConfig {
    fn default() -> Self {
        Self {
            push: true,
            pull: true,
            period: 5000,
            period_deviation: 5000,
            view_size: 30,
            healing_factor: 3,
            swapping_factor: 12,
        }
    }
}
