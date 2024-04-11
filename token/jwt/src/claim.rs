use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct Claim {
    id: ClaimId,
    exp: usize,
}

impl Claim {
    pub fn new(id: &ClaimId, exp: &usize) -> Self {
        Self { id: *id, exp: *exp }
    }

    pub fn id(&self) -> &ClaimId {
        &self.id
    }

    pub fn exp(&self) -> &usize {
        &self.exp
    }
}

#[derive(Deserialize, Serialize, Clone, Copy)]
pub enum ClaimId {
    Admin(Uuid),
    Token(Uuid, Option<UserClaim>),
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
