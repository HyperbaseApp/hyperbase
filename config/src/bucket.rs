use serde::Deserialize;

#[derive(Deserialize)]
pub struct BucketConfig {
    path: String,
}

impl BucketConfig {
    pub fn path(&self) -> &str {
        &self.path
    }
}
