use serde::Deserialize;

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

#[derive(Deserialize, Clone, Copy)]
pub enum SchemaFieldKind {
    Null,
    Integer,
    Real,
    Text,
    Blob,
}

impl SchemaFieldKind {
    pub fn to_str(&self) -> &str {
        match self {
            Self::Null => "null",
            Self::Integer => "integer",
            Self::Real => "real",
            Self::Text => "text",
            Self::Blob => "blob",
        }
    }

    pub fn from_str(str: &str) -> Result<Self, &str> {
        match str {
            "null" => Ok(Self::Null),
            "integer" => Ok(Self::Integer),
            "real" => Ok(Self::Real),
            "text" => Ok(Self::Text),
            "blob" => Ok(Self::Blob),
            _ => Err("Unknown schema field kind"),
        }
    }
}
