use std::collections::hash_map::Keys;

use ahash::{HashMap, HashMapExt};
use anyhow::{Error, Result};
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime};
use uuid::Uuid;

use crate::collection::SchemaFieldKind;

pub struct RecordDao {
    table_name: String,
    base: HashMap<String, Value>,
}

impl RecordDao {
    pub fn new(table_name: &str, capacity: &Option<usize>) -> Self {
        let capacity = match capacity {
            Some(capacity) => capacity + 1,
            None => 1,
        };

        let mut base = HashMap::with_capacity(capacity);
        base.insert("_id".to_owned(), Value::Uuid(Some(Uuid::new_v4())));

        Self {
            table_name: table_name.to_owned(),
            base,
        }
    }

    pub fn new_table_name(collection_id: &Uuid) -> String {
        "record_".to_owned() + &collection_id.to_string().replace("-", "")
    }

    pub fn table_name(&self) -> &str {
        &self.table_name
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.base.get(key)
    }

    pub fn keys(&self) -> Keys<'_, String, Value> {
        self.base.keys()
    }

    pub fn insert(&mut self, key: &str, value: &Value) {
        self.base.insert(key.to_owned(), value.to_owned());
    }
}

#[derive(Clone)]
pub enum Value {
    Boolean(Option<bool>),
    TinyInteger(Option<i8>),
    SmallInteger(Option<i16>),
    Integer(Option<i32>),
    BigInteger(Option<i64>),
    Float(Option<f32>),
    Double(Option<f64>),
    String(Option<String>),
    Byte(Option<Vec<u8>>),
    Uuid(Option<Uuid>),
    Date(Option<NaiveDate>),
    Time(Option<NaiveTime>),
    DateTime(Option<DateTime<FixedOffset>>),
    Timestamp(Option<DateTime<FixedOffset>>),
    Json(Option<Vec<u8>>),
}

impl Value {
    pub fn from_serde_json(kind: &SchemaFieldKind, value: &serde_json::Value) -> Result<Self> {
        match value {
            serde_json::Value::Null => Ok(Self::none(kind)),
            serde_json::Value::Bool(value) => match kind {
                SchemaFieldKind::Bool => Ok(Self::Boolean(Some(*value))),
                SchemaFieldKind::Byte => Ok(Self::Byte(Some(vec![*value as u8]))),
                _ => return Err(Error::msg("wrong value type")),
            },
            serde_json::Value::Number(value) => match kind {
                SchemaFieldKind::TinyInt => match value.as_i64() {
                    Some(value) => match i8::try_from(value) {
                        Ok(value) => Ok(Self::TinyInteger(Some(value))),
                        Err(err) => Err(err.into()),
                    },
                    None => Err(Error::msg("wrong value type")),
                },
                SchemaFieldKind::SmallInt => match value.as_i64() {
                    Some(value) => match i16::try_from(value) {
                        Ok(value) => Ok(Self::SmallInteger(Some(value))),
                        Err(err) => Err(err.into()),
                    },
                    None => Err(Error::msg("wrong value type")),
                },
                SchemaFieldKind::Int => match value.as_i64() {
                    Some(value) => match i32::try_from(value) {
                        Ok(value) => Ok(Self::Integer(Some(value))),
                        Err(err) => Err(err.into()),
                    },
                    None => Err(Error::msg("wrong value type")),
                },
                SchemaFieldKind::BigInt => match value.as_i64() {
                    Some(value) => Ok(Self::BigInteger(Some(value))),
                    None => Err(Error::msg("wrong value type")),
                },
                SchemaFieldKind::Float => match value.as_f64() {
                    Some(value) => {
                        let value = value as f32;
                        if value.is_finite() {
                            Ok(Self::Float(Some(value)))
                        } else {
                            Err(Error::msg("wrong value type"))
                        }
                    }
                    None => Err(Error::msg("wrong value type")),
                },
                SchemaFieldKind::Double => match value.as_f64() {
                    Some(value) => Ok(Self::Double(Some(value))),
                    None => Err(Error::msg("wrong value type")),
                },
                SchemaFieldKind::Byte => Ok(Self::Byte(Some(value.to_string().into_bytes()))),
                _ => return Err(Error::msg("wrong value type")),
            },
            serde_json::Value::String(value) => match kind {
                SchemaFieldKind::String => Ok(Self::String(Some(value.to_owned()))),
                SchemaFieldKind::Byte => Ok(Self::Byte(Some(value.as_bytes().to_vec()))),
                _ => return Err(Error::msg("wrong value type")),
            },
            serde_json::Value::Array(value) => match kind {
                SchemaFieldKind::Byte => {
                    let mut bytes = Vec::with_capacity(value.len());
                    for value in value.iter() {
                        match value.as_str() {
                            Some(value) => bytes.append(&mut value.as_bytes().to_vec()),
                            None => return Err(Error::msg("wrong value type")),
                        }
                    }
                    Ok(Self::Byte(Some(bytes)))
                }
                SchemaFieldKind::Json => {
                    let mut bytes: Vec<u8> = Vec::with_capacity(value.len());
                    for value in value.iter() {
                        match value.as_str() {
                            Some(value) => bytes.append(&mut value.as_bytes().to_vec()),
                            None => return Err(Error::msg("wrong value type")),
                        }
                    }
                    Ok(Self::Json(Some(bytes)))
                }
                _ => return Err(Error::msg("wrong value type")),
            },
            serde_json::Value::Object(value) => match kind {
                SchemaFieldKind::Byte => match serde_json::json!(value).as_str() {
                    Some(value) => Ok(Self::Byte(Some(value.as_bytes().to_vec()))),
                    None => return Err(Error::msg("wrong value type")),
                },
                SchemaFieldKind::Json => match serde_json::json!(value).as_str() {
                    Some(value) => Ok(Self::Json(Some(value.as_bytes().to_vec()))),
                    None => return Err(Error::msg("wrong value type")),
                },
                _ => return Err(Error::msg("wrong value type")),
            },
        }
    }
    pub fn none(kind: &SchemaFieldKind) -> Self {
        match kind {
            SchemaFieldKind::Bool => Self::Boolean(None),
            SchemaFieldKind::TinyInt => Self::TinyInteger(None),
            SchemaFieldKind::SmallInt => Self::SmallInteger(None),
            SchemaFieldKind::Int => Self::Integer(None),
            SchemaFieldKind::BigInt => Self::BigInteger(None),
            SchemaFieldKind::Float => Self::Float(None),
            SchemaFieldKind::Double => Self::Double(None),
            SchemaFieldKind::String => Self::String(None),
            SchemaFieldKind::Byte => Self::Byte(None),
            SchemaFieldKind::Uuid => Self::Uuid(None),
            SchemaFieldKind::Date => Self::Date(None),
            SchemaFieldKind::Time => Self::Time(None),
            SchemaFieldKind::DateTime => Self::DateTime(None),
            SchemaFieldKind::Timestamp => Self::Timestamp(None),
            SchemaFieldKind::Json => Self::Json(None),
        }
    }
}
