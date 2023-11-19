use std::str::FromStr;

use scylla::{
    cql_to_rust::{FromCqlVal, FromCqlValError},
    frame::{
        response::result::CqlValue,
        value::{Timestamp, Value, ValueTooBig},
    },
    BufMut, FromRow, FromUserType, IntoUserType, ValueList,
};
use strum::{Display, EnumString};
use uuid::Uuid;

#[derive(ValueList, FromRow)]
pub struct CollectionScyllaModel {
    id: Uuid,
    created_at: Timestamp,
    updated_at: Timestamp,
    project_id: Uuid,
    name: String,
    schema_fields: Vec<SchemaFieldScyllaModel>,
    indexes: Vec<String>,
}

impl CollectionScyllaModel {
    pub fn new(
        id: &Uuid,
        created_at: &Timestamp,
        updated_at: &Timestamp,
        project_id: &Uuid,
        name: &str,
        schema_fields: &Vec<SchemaFieldScyllaModel>,
        indexes: &Vec<String>,
    ) -> Self {
        Self {
            id: *id,
            created_at: *created_at,
            updated_at: *updated_at,
            project_id: *project_id,
            name: name.to_string(),
            schema_fields: schema_fields.to_vec(),
            indexes: indexes.to_vec(),
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

    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn schema_fields(&self) -> &Vec<SchemaFieldScyllaModel> {
        &self.schema_fields
    }

    pub fn indexes(&self) -> &Vec<String> {
        &self.indexes
    }
}

#[derive(IntoUserType, FromUserType, Clone)]
pub struct SchemaFieldScyllaModel {
    name: String,
    kind: SchemaFieldScyllaKind,
    required: bool,
}

impl SchemaFieldScyllaModel {
    pub fn new(name: &str, kind: &SchemaFieldScyllaKind, required: &bool) -> Self {
        Self {
            name: name.to_string(),
            kind: *kind,
            required: *required,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> &SchemaFieldScyllaKind {
        &self.kind
    }

    pub fn required(&self) -> &bool {
        &self.required
    }
}

#[derive(Display, EnumString, Clone, Copy)]
pub enum SchemaFieldScyllaKind {
    Boolean,
    Tinyint,
    Smallint,
    Int,
    Bigint,
    Float,
    Double,
    Ascii,
    Text,
    Varchar,
    Blob,
    Inet,
    Uuid,
    Timeuuid,
    Date,
    Time,
    Timestamp,
    Duration,
    Decimal,
    Varint,
    List,
    Set,
    Map,
    Tuple,
}

impl Value for SchemaFieldScyllaKind {
    fn serialize(&self, buf: &mut Vec<u8>) -> Result<(), ValueTooBig> {
        let kind = self.to_string();
        let str_bytes: &[u8] = kind.as_bytes();
        let val_len: i32 = str_bytes.len().try_into().map_err(|_| ValueTooBig)?;

        buf.put_i32(val_len);
        buf.put(str_bytes);

        Ok(())
    }
}

impl FromCqlVal<CqlValue> for SchemaFieldScyllaKind {
    fn from_cql(cql_val: CqlValue) -> Result<Self, FromCqlValError> {
        Ok(Self::from_str(&cql_val.as_text().unwrap()).unwrap())
    }
}
