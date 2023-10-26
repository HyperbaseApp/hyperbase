use std::time;

use anyhow::Result;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use uuid::Uuid;

use crate::{claim::Claim, kind::JwtTokenKind};

pub struct JwtToken {
    header: Header,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    expiry_duration: u64,
}

impl JwtToken {
    pub fn new(secret: &str, expiry_duration: &u64) -> Self {
        let secret = secret.as_bytes();
        Self {
            header: Header::default(),
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
            expiry_duration: *expiry_duration,
        }
    }

    pub fn encode(&self, id: &Uuid, kind: &JwtTokenKind) -> Result<String> {
        let expiration_time = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)?
            .as_secs()
            + self.expiry_duration;

        Ok(encode(
            &self.header,
            &Claim::new(id, kind, &(expiration_time as usize)),
            &self.encoding_key,
        )?)
    }

    pub fn decode(&self, token: &str) -> Result<Claim> {
        Ok(decode::<Claim>(token, &self.decoding_key, &Validation::default())?.claims)
    }
}
