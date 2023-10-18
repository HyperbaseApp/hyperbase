use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct PasswordBasedJson {
    #[validate(email)]
    email: String,
    password: String,
}

impl PasswordBasedJson {
    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn password(&self) -> &str {
        &self.password
    }
}

#[derive(Deserialize)]
pub struct TokenBasedJson {
    token: String,
}

impl TokenBasedJson {
    pub fn token(&self) -> &str {
        &self.token
    }
}

#[derive(Deserialize, Validate)]
pub struct RegisterJson {
    #[validate(email)]
    email: String,
    password: String,
}

impl RegisterJson {
    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn password(&self) -> &str {
        &self.password
    }
}

#[derive(Deserialize, Validate)]
pub struct RequestPasswordResetJson {
    #[validate(email)]
    email: String,
}

impl RequestPasswordResetJson {
    pub fn email(&self) -> &str {
        &self.email
    }
}

#[derive(Deserialize)]
pub struct ConfirmPasswordResetJson {
    code: String,
    password: String,
}

impl ConfirmPasswordResetJson {
    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn password(&self) -> &str {
        &self.password
    }
}
