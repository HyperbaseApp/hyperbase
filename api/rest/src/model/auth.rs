use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct RegisterReqJson {
    #[validate(email)]
    email: String,
    password: String,
}

impl RegisterReqJson {
    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn password(&self) -> &str {
        &self.password
    }
}

#[derive(Deserialize)]
pub struct VerifyRegistrationReqJson {
    id: Uuid,
    code: String,
}

impl VerifyRegistrationReqJson {
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn code(&self) -> &str {
        &self.code
    }
}

#[derive(Deserialize, Validate)]
pub struct PasswordBasedReqJson {
    #[validate(email)]
    email: String,
    password: String,
}

impl PasswordBasedReqJson {
    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn password(&self) -> &str {
        &self.password
    }
}

#[derive(Deserialize)]
pub struct TokenBasedReqJson {
    token: String,
}

impl TokenBasedReqJson {
    pub fn token(&self) -> &str {
        &self.token
    }
}

#[derive(Deserialize, Validate)]
pub struct RequestPasswordResetReqJson {
    #[validate(email)]
    email: String,
}

impl RequestPasswordResetReqJson {
    pub fn email(&self) -> &str {
        &self.email
    }
}

#[derive(Deserialize)]
pub struct ConfirmPasswordResetReqJson {
    id: Uuid,
    code: String,
    password: String,
}

impl ConfirmPasswordResetReqJson {
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn password(&self) -> &str {
        &self.password
    }
}

#[derive(Serialize)]
pub struct RegisterResJson {
    id: Uuid,
}

impl RegisterResJson {
    pub fn new(id: &Uuid) -> Self {
        Self { id: *id }
    }
}

#[derive(Serialize)]
pub struct VerifyRegistrationResJson {
    id: Uuid,
}

impl VerifyRegistrationResJson {
    pub fn new(id: &Uuid) -> Self {
        Self { id: *id }
    }
}

#[derive(Serialize)]
pub struct AuthTokenResJson {
    token: String,
}

impl AuthTokenResJson {
    pub fn new(token: &str) -> Self {
        Self {
            token: token.to_owned(),
        }
    }
}

#[derive(Serialize)]
pub struct RequestPasswordResetResJson {
    id: Uuid,
}

impl RequestPasswordResetResJson {
    pub fn new(id: &Uuid) -> Self {
        Self { id: *id }
    }
}

#[derive(Serialize)]
pub struct ConfirmPasswordResetResJson {
    id: Uuid,
}

impl ConfirmPasswordResetResJson {
    pub fn new(id: &Uuid) -> Self {
        Self { id: *id }
    }
}
