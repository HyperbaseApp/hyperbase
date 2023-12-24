use std::any::type_name;

use scylla::{
    cql_to_rust::{FromCqlVal, FromCqlValError},
    frame::response::result::{ColumnType, CqlValue},
    serialize::{
        value::{BuiltinSerializationError, BuiltinSerializationErrorKind, SerializeCql},
        writers::WrittenCellProof,
        CellWriter, SerializationError,
    },
};

pub const LOGICAL_OPERATOR: [&str; 1] = ["AND"];

pub const COMPARISON_OPERATOR: [&str; 8] =
    ["=", "<", ">", "<=", ">=", "IN", "CONTAINS", "CONTAINS KEY"];

pub const ORDER_TYPE: [&str; 2] = ["ASC", "DESC"];

#[derive(Clone, Copy)]
pub enum SchemaFieldKind {
    Ascii,
    Boolean,
    Blob,
    Counter,
    Decimal,
    Date,
    Double,
    Duration,
    Empty,
    Float,
    Int,
    BigInt,
    Text,
    Timestamp,
    Inet,
    List,
    Map,
    Set,
    UserDefinedType,
    SmallInt,
    TinyInt,
    Time,
    Timeuuid,
    Tuple,
    Uuid,
    Varint,
}

impl SchemaFieldKind {
    pub fn to_str(&self) -> &str {
        match self {
            Self::Ascii => "ascii",
            Self::Boolean => "boolean",
            Self::Blob => "blob",
            Self::Counter => "counter",
            Self::Decimal => "decimal",
            Self::Date => "date",
            Self::Double => "double",
            Self::Duration => "duration",
            Self::Empty => "empty",
            Self::Float => "float",
            Self::Int => "int",
            Self::BigInt => "bigint",
            Self::Text => "text",
            Self::Timestamp => "timestamp",
            Self::Inet => "inet",
            Self::List => "list",
            Self::Map => "map",
            Self::Set => "set",
            Self::UserDefinedType => "userdefinedtype",
            Self::SmallInt => "smallint",
            Self::TinyInt => "tinyint",
            Self::Time => "time",
            Self::Timeuuid => "timeuuid",
            Self::Tuple => "tuple",
            Self::Uuid => "uuid",
            Self::Varint => "varint",
        }
    }

    pub fn from_str(str: &str) -> Result<Self, String> {
        match str {
            "ascii" => Ok(Self::Ascii),
            "boolean" => Ok(Self::Boolean),
            "blob" => Ok(Self::Blob),
            "counter" => Ok(Self::Counter),
            "decimal" => Ok(Self::Decimal),
            "date" => Ok(Self::Date),
            "double" => Ok(Self::Double),
            "duration" => Ok(Self::Duration),
            "empty" => Ok(Self::Empty),
            "float" => Ok(Self::Float),
            "int" => Ok(Self::Int),
            "bigint" => Ok(Self::BigInt),
            "text" => Ok(Self::Text),
            "timestamp" => Ok(Self::Timestamp),
            "inet" => Ok(Self::Inet),
            "list" => Ok(Self::List),
            "map" => Ok(Self::Map),
            "set" => Ok(Self::Set),
            "userdefinedtype" => Ok(Self::UserDefinedType),
            "smallint" => Ok(Self::SmallInt),
            "tinyint" => Ok(Self::TinyInt),
            "time" => Ok(Self::Time),
            "timeuuid" => Ok(Self::Timeuuid),
            "tuple" => Ok(Self::Tuple),
            "uuid" => Ok(Self::Uuid),
            "varint" => Ok(Self::Varint),
            _ => Err(format!("Unknown schema field kind '{str}'")),
        }
    }
}

impl FromCqlVal<CqlValue> for SchemaFieldKind {
    fn from_cql(cql_val: CqlValue) -> Result<Self, FromCqlValError> {
        Ok(Self::from_str(&cql_val.as_text().unwrap()).unwrap())
    }
}

impl SerializeCql for SchemaFieldKind {
    fn serialize<'a>(
        &self,
        typ: &ColumnType,
        writer: CellWriter<'a>,
    ) -> Result<WrittenCellProof<'a>, SerializationError> {
        writer.set_value(self.to_str().as_bytes()).map_err(|_| {
            SerializationError::new(BuiltinSerializationError {
                rust_name: type_name::<&str>(),
                got: typ.clone(),
                kind: BuiltinSerializationErrorKind::SizeOverflow,
            })
        })
    }
}
