use serde::{Deserialize, Serialize};

pub const LOGICAL_OPERATOR: [&str; 2] = ["AND", "OR"];

pub const COMPARISON_OPERATOR: [&str; 18] = [
    ">",
    ">=",
    "<",
    "<=",
    "<>",
    "!=",
    "<=>",
    "=",
    "BETWEEN",
    "NOT BETWEEN",
    "IS",
    "IS NOT",
    "IS NULL",
    "IS NOT NULL",
    "IN",
    "NOT IN",
    "LIKE",
    "NOT LIKE",
];

pub const ORDER_TYPE: [&str; 2] = ["ASC", "DESC"];

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum SchemaFieldKind {
    Boolean,
    Integer,
    Bigint,
    Int8,
    Real,
    Text,
    Blob,
    Datetime,
    Date,
    Time,
}

impl SchemaFieldKind {
    pub fn to_str(&self) -> &str {
        match self {
            Self::Boolean => "boolean",
            Self::Integer => "integer",
            Self::Bigint => "bigint",
            Self::Int8 => "int8",
            Self::Real => "real",
            Self::Text => "text",
            Self::Blob => "blob",
            Self::Datetime => "datetime",
            Self::Date => "date",
            Self::Time => "time",
        }
    }

    pub fn from_str(str: &str) -> Result<Self, String> {
        match str {
            "boolean" => Ok(Self::Boolean),
            "integer" => Ok(Self::Integer),
            "bigint" => Ok(Self::Bigint),
            "int8" => Ok(Self::Int8),
            "real" => Ok(Self::Real),
            "text" => Ok(Self::Text),
            "blob" => Ok(Self::Blob),
            "datetime" => Ok(Self::Datetime),
            "date" => Ok(Self::Date),
            "time" => Ok(Self::Time),
            _ => Err(format!("Unknown schema field kind '{str}'")),
        }
    }
}
