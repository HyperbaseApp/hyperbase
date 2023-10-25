use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum JwtTokenKind {
    Admin,
    Token,
}
