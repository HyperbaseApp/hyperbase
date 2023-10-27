use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::kind::JwtTokenKind;

#[derive(Serialize, Deserialize)]
pub struct Claim {
    id: Uuid,
    kind: JwtTokenKind,
    exp: usize,
}

impl Claim {
    pub fn new(id: &Uuid, kind: &JwtTokenKind, exp: &usize) -> Self {
        Self {
            id: *id,
            kind: *kind,
            exp: *exp,
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn kind(&self) -> &JwtTokenKind {
        &self.kind
    }
}
