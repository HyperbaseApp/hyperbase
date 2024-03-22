use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, PartialEq, Clone, Copy)]
pub enum JwtTokenKind {
    Admin,
    UserAnonymous,
    User,
}
