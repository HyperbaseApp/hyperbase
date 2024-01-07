use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, PartialEq, Clone, Copy)]
pub enum JwtTokenKind {
    User,
    Token,
}
