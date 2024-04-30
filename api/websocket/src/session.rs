use crate::{AdminId, TokenId, UserId};

pub enum UserSession {
    Admin(AdminId),
    Token(TokenId, Option<UserId>),
}
