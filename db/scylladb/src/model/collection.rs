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
    schema_fields: Vec<SchemaScyllaFieldModel>,
    indexes: Vec<String>,
}

impl CollectionScyllaModel {
    pub fn new(
        id: Uuid,
        created_at: Timestamp,
        updated_at: Timestamp,
        project_id: Uuid,
        name: String,
        schema_fields: Vec<SchemaScyllaFieldModel>,
        indexes: Vec<String>,
    ) -> Self {
        Self {
            id,
            created_at,
            updated_at,
            project_id,
            name,
            schema_fields,
            indexes,
        }
    }
}

impl CollectionScyllaModel {
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

    pub fn schema_fields(&self) -> &Vec<SchemaScyllaFieldModel> {
        &self.schema_fields
    }

    pub fn indexes(&self) -> &Vec<String> {
        &self.indexes
    }
}

#[derive(IntoUserType, FromUserType)]
pub struct SchemaScyllaFieldModel {
    name: String,
    kind: SchemaScyllaFieldKind,
    required: bool,
}

impl SchemaScyllaFieldModel {
    pub fn new(name: String, kind: SchemaScyllaFieldKind, required: bool) -> Self {
        Self {
            name,
            kind,
            required,
        }
    }
}

impl SchemaScyllaFieldModel {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> &SchemaScyllaFieldKind {
        &self.kind
    }

    pub fn required(&self) -> &bool {
        &self.required
    }
}

#[derive(Display, EnumString)]
pub enum SchemaScyllaFieldKind {
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

impl Value for SchemaScyllaFieldKind {
    fn serialize(&self, buf: &mut Vec<u8>) -> Result<(), ValueTooBig> {
        let kind = self.to_string();
        let str_bytes: &[u8] = kind.as_bytes();
        let val_len: i32 = str_bytes.len().try_into().map_err(|_| ValueTooBig)?;

        buf.put_i32(val_len);
        buf.put(str_bytes);

        Ok(())
    }
}

impl FromCqlVal<CqlValue> for SchemaScyllaFieldKind {
    fn from_cql(cql_val: CqlValue) -> Result<Self, FromCqlValError> {
        Ok(Self::from_str(&cql_val.as_text().unwrap()).unwrap())
    }
}
