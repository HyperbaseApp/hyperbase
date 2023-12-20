use serde::Deserialize;

pub const LOGICAL_OPERATOR: [&str; 2] = ["AND", "OR"];

pub const COMPARISON_OPERATOR: [&str; 19] = [
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
    "ISNULL",
    "IN",
    "NOT IN",
    "LIKE",
    "NOT LIKE",
];

pub const ORDER_TYPE: [&str; 2] = ["ASC", "DESC"];

#[derive(Deserialize, Clone, Copy)]
pub enum SchemaFieldKind {
    Char,
    Varchar,
    Tinyint,
    Smallint,
    Int,
    Bigint,
    Float,
    Double,
    Bool,
    Blob,
    Date,
    Time,
    Datetime,
    Timestamp,
    Binary16,
    Json,
    Decimal,
}

impl SchemaFieldKind {
    pub fn to_str(&self) -> &str {
        match self {
            Self::Char => "char",
            Self::Varchar => "varchar",
            Self::Tinyint => "tinyint",
            Self::Smallint => "smallint",
            Self::Int => "int",
            Self::Bigint => "bigint",
            Self::Float => "float",
            Self::Double => "double",
            Self::Bool => "bool",
            Self::Blob => "blob",
            Self::Date => "date",
            Self::Time => "time",
            Self::Datetime => "datetime",
            Self::Timestamp => "timestamp",
            Self::Binary16 => "binary16",
            Self::Json => "json",
            Self::Decimal => "decimal",
        }
    }

    pub fn from_str(str: &str) -> Result<Self, &str> {
        match str {
            "char" => Ok(Self::Char),
            "varchar" => Ok(Self::Varchar),
            "tinyint" => Ok(Self::Tinyint),
            "smallint" => Ok(Self::Smallint),
            "int" => Ok(Self::Int),
            "bigint" => Ok(Self::Bigint),
            "float" => Ok(Self::Float),
            "double" => Ok(Self::Double),
            "bool" => Ok(Self::Bool),
            "blob" => Ok(Self::Blob),
            "date" => Ok(Self::Date),
            "time" => Ok(Self::Time),
            "datetime" => Ok(Self::Datetime),
            "timestamp" => Ok(Self::Timestamp),
            "binary16" => Ok(Self::Binary16),
            "json" => Ok(Self::Json),
            "decimal" => Ok(Self::Decimal),
            _ => Err("Unknown schema field kind"),
        }
    }
}
