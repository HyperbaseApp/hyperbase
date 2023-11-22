use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum JwtTokenKind {
    User,
    Token,
}
