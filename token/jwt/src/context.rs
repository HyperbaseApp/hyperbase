use jsonwebtoken::{
    decode, encode, errors::Error, Algorithm, DecodingKey, EncodingKey, Header, TokenData,
    Validation,
};
use uuid::Uuid;

use crate::{claim::Claim, kind::JwtTokenKind};

pub struct JwtToken {
    header: Header,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    expiration_time: usize,
}

impl JwtToken {
    pub fn new(secret: &str, expiration_time: &usize) -> Self {
        let secret = secret.as_bytes();
        Self {
            header: Header::new(Algorithm::EdDSA),
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
            expiration_time: *expiration_time,
        }
    }

    pub fn encode(&self, id: &Uuid, kind: &JwtTokenKind) -> Result<String, Error> {
        encode(
            &self.header,
            &Claim::new(id, kind, &self.expiration_time),
            &self.encoding_key,
        )
    }

    pub fn decode(&self, token: &str) -> Result<Claim, Error> {
        Ok(decode::<Claim>(token, &self.decoding_key, &Validation::default())?.claims)
    }
}
