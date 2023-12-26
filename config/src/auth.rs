use serde::Deserialize;

#[derive(Deserialize)]
pub struct AuthConfig {
    admin_registration: bool,
    access_token_length: usize,
    registration_ttl: u32,
    reset_password_ttl: u32,
}

impl AuthConfig {
    pub fn admin_registration(&self) -> &bool {
        &self.admin_registration
    }

    pub fn access_token_length(&self) -> &usize {
        &self.access_token_length
    }

    pub fn registration_ttl(&self) -> &u32 {
        &self.registration_ttl
    }

    pub fn reset_password_ttl(&self) -> &u32 {
        &self.reset_password_ttl
    }
}
