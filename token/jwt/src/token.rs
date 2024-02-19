use std::time;

use anyhow::Result;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use uuid::Uuid;

use crate::{
    claim::{Claim, UserClaim},
    kind::JwtTokenKind,
};

pub struct JwtToken {
    header: Header,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    expiry_duration: u64,
}

impl JwtToken {
    pub fn new(secret: &str, expiry_duration: &u64) -> Self {
        hb_log::info(Some("⚡"), "JwtToken: Initializing component");

        let secret = secret.as_bytes();
        Self {
            header: Header::default(),
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
            expiry_duration: *expiry_duration,
        }
    }

    pub fn encode(
        &self,
        id: &Uuid,
        user: &Option<UserClaim>,
        kind: &JwtTokenKind,
    ) -> Result<String> {
        let expiration_time = match usize::try_from(
            time::SystemTime::now()
                .duration_since(time::UNIX_EPOCH)?
                .as_secs()
                + self.expiry_duration,
        ) {
            Ok(time) => time,
            Err(err) => return Err(err.into()),
        };

        Ok(encode(
            &self.header,
            &Claim::new(id, user, kind, &expiration_time),
            &self.encoding_key,
        )?)
    }

    pub fn decode(&self, token: &str) -> Result<Claim> {
        Ok(decode::<Claim>(token, &self.decoding_key, &Validation::default())?.claims)
    }

    pub fn need_renew(&self, claim: &Claim) -> Result<bool> {
        let expiry = match u64::try_from(*claim.exp()) {
            Ok(expiry) => expiry,
            Err(err) => return Err(err.into()),
        };
        if expiry - (self.expiry_duration / 2)
            < time::SystemTime::now()
                .duration_since(time::UNIX_EPOCH)?
                .as_secs()
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn renew(&self, claim: &Claim) -> Result<String> {
        self.encode(claim.id(), claim.user(), claim.kind())
    }
}
