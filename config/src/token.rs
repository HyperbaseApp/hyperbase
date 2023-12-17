use serde::Deserialize;

#[derive(Deserialize)]
pub struct TokenConfig {
    jwt: JwtTokenConfig,
}

impl TokenConfig {
    pub fn jwt(&self) -> &JwtTokenConfig {
        &self.jwt
    }
}

#[derive(Deserialize)]
pub struct JwtTokenConfig {
    secret: String,
    expiry_duration: u64,
}

impl JwtTokenConfig {
    pub fn secret(&self) -> &str {
        &self.secret
    }

    pub fn expiry_duration(&self) -> &u64 {
        &self.expiry_duration
    }
}
