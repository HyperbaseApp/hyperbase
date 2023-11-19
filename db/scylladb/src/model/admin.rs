use std::str::FromStr;

use scylla::{
    cql_to_rust::FromCqlVal,
    frame::{
        response::result::CqlValue,
        value::{Timestamp, Value, ValueTooBig},
    },
    BufMut, FromRow, ValueList,
};
use strum::{Display, EnumString};
use uuid::Uuid;

#[derive(ValueList, FromRow)]
pub struct AdminScyllaModel {
    id: Uuid,
    created_at: Timestamp,
    updated_at: Timestamp,
    email: String,
    password_hash: String,
    role: AdminScyllaRole,
}

impl AdminScyllaModel {
    pub fn new(
        id: &Uuid,
        created_at: &Timestamp,
        updated_at: &Timestamp,
        email: &str,
        password_hash: &str,
        role: &AdminScyllaRole,
    ) -> Self {
        Self {
            id: *id,
            created_at: *created_at,
            updated_at: *updated_at,
            email: email.to_string(),
            password_hash: password_hash.to_string(),
            role: *role,
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn created_at(&self) -> &Timestamp {
        &self.created_at
    }

    pub fn updated_at(&self) -> &Timestamp {
        &self.updated_at
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn password_hash(&self) -> &str {
        &self.password_hash
    }

    pub fn role(&self) -> &AdminScyllaRole {
        &self.role
    }
}

#[derive(Display, EnumString, Clone, Copy)]
pub enum AdminScyllaRole {
    SuperUser,
    User,
}

impl Value for AdminScyllaRole {
    fn serialize(&self, buf: &mut Vec<u8>) -> Result<(), scylla::frame::value::ValueTooBig> {
        let role = self.to_string();
        let str_bytes: &[u8] = role.as_bytes();
        let val_len: i32 = str_bytes.len().try_into().map_err(|_| ValueTooBig)?;

        buf.put_i32(val_len);
        buf.put(str_bytes);

        Ok(())
    }
}

impl FromCqlVal<CqlValue> for AdminScyllaRole {
    fn from_cql(cql_val: CqlValue) -> Result<Self, scylla::cql_to_rust::FromCqlValError> {
        Ok(Self::from_str(&cql_val.as_text().unwrap()).unwrap())
    }
}
