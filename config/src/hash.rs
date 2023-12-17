use serde::Deserialize;

#[derive(Deserialize)]
pub struct HashConfig {
    argon2: Argon2HashConfig,
}

impl HashConfig {
    pub fn argon2(&self) -> &Argon2HashConfig {
        &self.argon2
    }
}

#[derive(Deserialize)]
pub struct Argon2HashConfig {
    algorithm: String,
    version: String,
    salt: String,
}

impl Argon2HashConfig {
    pub fn algorithm(&self) -> &str {
        &self.algorithm
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn salt(&self) -> &str {
        &self.salt
    }
}
