use scylla::{
    cql_to_rust::{FromCqlVal, FromCqlValError},
    frame::{
        response::result::CqlValue,
        value::{Value, ValueTooBig},
    },
    BufMut,
};

pub const LOGICAL_OPERATOR: [&str; 1] = ["AND"];

pub const COMPARISON_OPERATOR: [&str; 8] =
    ["=", "<", ">", "<=", ">=", "IN", "CONTAINS", "CONTAINS KEY"];

pub const ORDER_TYPE: [&str; 2] = ["ASC", "DESC"];

#[derive(Clone, Copy)]
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

impl SchemaFieldScyllaKind {
    pub fn to_str(&self) -> &str {
        match self {
            Self::Boolean => "boolean",
            Self::Tinyint => "tinyint",
            Self::Smallint => "smallint",
            Self::Int => "int",
            Self::Bigint => "bigint",
            Self::Float => "float",
            Self::Double => "double",
            Self::Ascii => "ascii",
            Self::Text => "text",
            Self::Varchar => "varchar",
            Self::Blob => "blob",
            Self::Inet => "inet",
            Self::Uuid => "uuid",
            Self::Timeuuid => "timeuuid",
            Self::Date => "date",
            Self::Time => "time",
            Self::Timestamp => "timestamp",
            Self::Duration => "duration",
            Self::Decimal => "decimal",
            Self::Varint => "varint",
            Self::List => "list",
            Self::Set => "set",
            Self::Map => "map",
            Self::Tuple => "tuple",
        }
    }

    pub fn from_str(str: &str) -> Result<Self, &str> {
        match str {
            "boolean" => Ok(Self::Boolean),
            "tinyint" => Ok(Self::Tinyint),
            "smallint" => Ok(Self::Smallint),
            "int" => Ok(Self::Int),
            "bigint" => Ok(Self::Bigint),
            "float" => Ok(Self::Float),
            "double" => Ok(Self::Double),
            "ascii" => Ok(Self::Ascii),
            "text" => Ok(Self::Text),
            "varchar" => Ok(Self::Varchar),
            "blob" => Ok(Self::Blob),
            "inet" => Ok(Self::Inet),
            "uuid" => Ok(Self::Uuid),
            "timeuuid" => Ok(Self::Timeuuid),
            "date" => Ok(Self::Date),
            "time" => Ok(Self::Time),
            "timestamp" => Ok(Self::Timestamp),
            "duration" => Ok(Self::Duration),
            "decimal" => Ok(Self::Decimal),
            "varint" => Ok(Self::Varint),
            "list" => Ok(Self::List),
            "set" => Ok(Self::Set),
            "map" => Ok(Self::Map),
            "tuple" => Ok(Self::Tuple),
            _ => Err("Unknown schema field kind"),
        }
    }
}

impl Value for SchemaFieldScyllaKind {
    fn serialize(&self, buf: &mut Vec<u8>) -> Result<(), ValueTooBig> {
        let kind = self.to_str();
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
