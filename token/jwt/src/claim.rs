use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::kind::JwtTokenKind;

#[derive(Deserialize, Serialize)]
pub struct Claim {
    id: Uuid,
    user: Option<UserClaim>,
    kind: JwtTokenKind,
    exp: usize,
}

impl Claim {
    pub fn new(id: &Uuid, user: &Option<UserClaim>, kind: &JwtTokenKind, exp: &usize) -> Self {
        Self {
            id: *id,
            user: *user,
            kind: *kind,
            exp: *exp,
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn user(&self) -> &Option<UserClaim> {
        &self.user
    }

    pub fn kind(&self) -> &JwtTokenKind {
        &self.kind
    }

    pub fn exp(&self) -> &usize {
        &self.exp
    }
}

#[derive(Deserialize, Serialize, Clone, Copy)]
pub struct UserClaim {
    collection_id: Uuid,
    id: Uuid,
}

impl UserClaim {
    pub fn new(collection_id: &Uuid, id: &Uuid) -> Self {
        Self {
            collection_id: *collection_id,
            id: *id,
        }
    }

    pub fn collection_id(&self) -> &Uuid {
        &self.collection_id
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }
}
