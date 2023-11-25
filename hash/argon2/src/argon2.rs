use argon2::{
    password_hash::{self, SaltString},
    Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version,
};

pub struct Argon2Hash {
    argon2: Argon2<'static>,
    salt: SaltString,
}

impl Argon2Hash {
    pub fn new(algorithm: &str, version: &str, salt: &str) -> Self {
        hb_log::info(Some("⚡"), "Argon2Hash: Initializing component");

        let algorithm = match algorithm {
            "Argon2d" => Algorithm::Argon2d,
            "Argon2i" => Algorithm::Argon2i,
            "Argon2id" => Algorithm::Argon2id,
            _ => panic!("Unknown argon2 algorithm"),
        };

        let version = match version {
            "V0x10" => Version::V0x10,
            "V0x13" => Version::V0x13,
            _ => panic!("Unknown argon2 version"),
        };

        Self {
            argon2: Argon2::new(algorithm, version, Params::DEFAULT),
            salt: SaltString::from_b64(salt).unwrap(),
        }
    }

    pub fn hash_password(&self, password: &[u8]) -> Result<PasswordHash<'_>, password_hash::Error> {
        self.argon2.hash_password(password, &self.salt)
    }

    pub fn verify_password(&self, password: &str, hash: &str) -> Result<(), password_hash::Error> {
        let hash = PasswordHash::new(hash)?;
        self.argon2.verify_password(password.as_bytes(), &hash)
    }
}
