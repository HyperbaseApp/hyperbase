use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::kind::JwtTokenKind;

#[derive(Serialize, Deserialize)]
pub struct Claim {
    id: Uuid,
    role: Option<String>,
    kind: JwtTokenKind,
    exp: usize,
}

impl Claim {
    pub fn new(id: &Uuid, role: &Option<String>, kind: &JwtTokenKind, exp: &usize) -> Self {
        Self {
            id: *id,
            role: role.to_owned(),
            kind: *kind,
            exp: *exp,
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn role(&self) -> &Option<String> {
        &self.role
    }

    pub fn kind(&self) -> &JwtTokenKind {
        &self.kind
    }

    pub fn exp(&self) -> &usize {
        &self.exp
    }
}
