use serde::{Deserialize, Serialize};

pub const LOGICAL_OPERATOR: [&str; 2] = ["AND", "OR"];

pub const COMPARISON_OPERATOR: [&str; 29] = [
    "<",
    ">",
    "<=",
    ">=",
    "=",
    "<>",
    "!=",
    "BETWEEN",
    "NOT BETWEEN",
    "BETWEEN SYMMETRIC",
    "NOT BETWEEN SYMMETRIC",
    "IS DISTINCT FROM",
    "IS NOT DISTINCT FROM",
    "IS",
    "IS NOT",
    "IS NULL",
    "IS NOT NULL",
    "ISNULL",
    "NOTNULL",
    "IS TRUE",
    "IS NOT TRUE",
    "IS FALSE",
    "IS NOT FALSE",
    "IS UNKNOWN",
    "IS NOT UNKNOWN",
    "IN",
    "NOT IN",
    "LIKE",
    "NOT LIKE",
];

pub const ORDER_TYPE: [&str; 2] = ["ASC", "DESC"];

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum SchemaFieldKind {
    Bool,
    Char,
    Smallint,
    Smallserial,
    Int2,
    Integer,
    Serial,
    Int4,
    Bigint,
    Bigserial,
    Int8,
    Real,
    Float4,
    DoublePrecision,
    Float8,
    Numeric,
    Varchar,
    Text,
    Name,
    Citext,
    Bytea,
    Void,
    Timestamptz,
    Timestamp,
    Date,
    Time,
    Timetz,
    Uuid,
    Inet,
    Cidr,
    Macaddr,
    Bit,
    Varbit,
    Json,
    Jsonb,
    Arrays,
    Interval,
    Int8range,
    Int4range,
    Tsrange,
    Tstzrange,
    Daterange,
    Numrange,
    Money,
    Ltree,
    Lquery,
    Citext_,
}

impl SchemaFieldKind {
    pub fn to_str(&self) -> &str {
        match self {
            Self::Bool => "bool",
            Self::Char => "\"char\"",
            Self::Smallint => "smallint",
            Self::Smallserial => "smallserial",
            Self::Int2 => "int2",
            Self::Integer => "integer",
            Self::Serial => "serial",
            Self::Int4 => "int4",
            Self::Bigint => "bigint",
            Self::Bigserial => "bigserial",
            Self::Int8 => "int8",
            Self::Real => "real",
            Self::Float4 => "float4",
            Self::DoublePrecision => "double precision",
            Self::Float8 => "float8",
            Self::Numeric => "numeric",
            Self::Varchar => "varchar",
            Self::Text => "text",
            Self::Name => "name",
            Self::Citext => "citext",
            Self::Bytea => "bytea",
            Self::Void => "void",
            Self::Timestamptz => "timestamptz",
            Self::Timestamp => "timestamp",
            Self::Date => "date",
            Self::Time => "time",
            Self::Timetz => "timetz",
            Self::Uuid => "uuid",
            Self::Inet => "inet",
            Self::Cidr => "cidr",
            Self::Macaddr => "macaddr",
            Self::Bit => "bit",
            Self::Varbit => "varbit",
            Self::Json => "json",
            Self::Jsonb => "jsonb",
            Self::Arrays => "arrays",
            Self::Interval => "interval",
            Self::Int8range => "int8range",
            Self::Int4range => "int4range",
            Self::Tsrange => "tsrange",
            Self::Tstzrange => "tstzrange",
            Self::Daterange => "daterange",
            Self::Numrange => "numrange",
            Self::Money => "money",
            Self::Ltree => "ltree",
            Self::Lquery => "lquery",
            Self::Citext_ => "citext_",
        }
    }

    pub fn from_str(str: &str) -> Result<Self, String> {
        match str {
            "bool" => Ok(Self::Bool),
            "char" => Ok(Self::Char),
            "smallint" => Ok(Self::Smallint),
            "smallserial" => Ok(Self::Smallserial),
            "int2" => Ok(Self::Int2),
            "integer" => Ok(Self::Integer),
            "serial" => Ok(Self::Serial),
            "int4" => Ok(Self::Int4),
            "bigint" => Ok(Self::Bigint),
            "bigserial" => Ok(Self::Bigserial),
            "int8" => Ok(Self::Int8),
            "real" => Ok(Self::Real),
            "float4" => Ok(Self::Float4),
            "double precision" => Ok(Self::DoublePrecision),
            "float8" => Ok(Self::Float8),
            "numeric" => Ok(Self::Numeric),
            "varchar" => Ok(Self::Varchar),
            "text" => Ok(Self::Text),
            "name" => Ok(Self::Name),
            "citext" => Ok(Self::Citext),
            "bytea" => Ok(Self::Bytea),
            "void" => Ok(Self::Void),
            "timestamptz" => Ok(Self::Timestamptz),
            "timestamp" => Ok(Self::Timestamp),
            "date" => Ok(Self::Date),
            "time" => Ok(Self::Time),
            "timetz" => Ok(Self::Timetz),
            "uuid" => Ok(Self::Uuid),
            "inet" => Ok(Self::Inet),
            "cidr" => Ok(Self::Cidr),
            "macaddr" => Ok(Self::Macaddr),
            "bit" => Ok(Self::Bit),
            "varbit" => Ok(Self::Varbit),
            "json" => Ok(Self::Json),
            "jsonb" => Ok(Self::Jsonb),
            "arrays" => Ok(Self::Arrays),
            "interval" => Ok(Self::Interval),
            "int8range" => Ok(Self::Int8range),
            "int4range" => Ok(Self::Int4range),
            "tsrange" => Ok(Self::Tsrange),
            "tstzrange" => Ok(Self::Tstzrange),
            "daterange" => Ok(Self::Daterange),
            "numrange" => Ok(Self::Numrange),
            "money" => Ok(Self::Money),
            "ltree" => Ok(Self::Ltree),
            "lquery" => Ok(Self::Lquery),
            "citext_" => Ok(Self::Citext_),
            _ => Err(format!("Unknown schema field kind '{str}'")),
        }
    }
}
