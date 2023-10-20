use argon2::{
    password_hash::{self, Salt},
    Algorithm, Argon2, Params, PasswordHash, PasswordHasher, Version,
};
use hb_config::Argon2HashConfig;

pub struct Argon2Hash {
    argon2: Argon2<'static>,
    salt: String,
}

impl Argon2Hash {
    pub fn hash_password(&self, password: &[u8]) -> Result<PasswordHash<'_>, password_hash::Error> {
        self.argon2
            .hash_password(password, Salt::from_b64(&self.salt).unwrap())
    }
}

pub fn new(config: &Argon2HashConfig) -> Argon2Hash {
    let algorithm = match config.algorithm() {
        "Argon2d" => Algorithm::Argon2d,
        "Argon2i" => Algorithm::Argon2i,
        "Argon2id" => Algorithm::Argon2id,
        _ => panic!("Unknown argon2 algorithm"),
    };

    let version = match config.version() {
        "V0x10" => Version::V0x10,
        "V0x13" => Version::V0x13,
        _ => panic!("Unknown argon2 version"),
    };

    Argon2Hash {
        argon2: Argon2::new(algorithm, version, Params::DEFAULT),
        salt: config.salt().to_string(),
    }
}
