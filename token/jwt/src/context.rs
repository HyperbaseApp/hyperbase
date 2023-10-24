use jsonwebtoken::{encode, errors::Error, Algorithm, EncodingKey, Header};

use crate::claim::Claim;

pub struct JwtContext {
    header: Header,
    encoding_key: EncodingKey,
}

impl JwtContext {
    pub fn new(secret: &str) -> Self {
        Self {
            header: Header::new(Algorithm::EdDSA),
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
        }
    }

    pub fn encode(&self, claim: Claim) -> Result<String, Error> {
        encode(&self.header, &claim, &self.encoding_key)
    }
}
