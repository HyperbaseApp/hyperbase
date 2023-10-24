use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct Claim {
    id: Uuid,
    exp: usize,
}

impl Claim {
    pub fn new(id: &Uuid, exp: &usize) -> Self {
        Self { id: *id, exp: *exp }
    }
}
