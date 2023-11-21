use std::collections::HashMap;

use anyhow::{Error, Result};
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime};
use uuid::Uuid;

use crate::{collection::SchemaFieldKind, Db};

pub struct RecordDao {
    data: HashMap<String, Value>,
}

impl RecordDao {
    pub fn new(capacity: Option<&usize>) -> Self {
        match capacity {
            Some(capacity) => Self {
                data: HashMap::with_capacity(*capacity),
            },
            None => Self {
                data: HashMap::new(),
            },
        }
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }

    pub fn insert(&mut self, key: &str, value: &Value) {
        self.data.insert(key.to_string(), value.to_owned());
    }

    pub async fn db_insert(&self, db: &Db) -> Result<()> {
        Ok(())
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
            serde_json::Value::Null => match kind {
                SchemaFieldKind::Boolean => Ok(Self::Boolean(None)),
                SchemaFieldKind::TinyInteger => Ok(Self::TinyInteger(None)),
                SchemaFieldKind::SmallInteger => Ok(Self::SmallInteger(None)),
                SchemaFieldKind::Integer => Ok(Self::Integer(None)),
                SchemaFieldKind::BigInteger => Ok(Self::BigInteger(None)),
                SchemaFieldKind::Float => Ok(Self::Float(None)),
                SchemaFieldKind::Double => Ok(Self::Double(None)),
                SchemaFieldKind::String => Ok(Self::String(None)),
                SchemaFieldKind::Byte => Ok(Self::Byte(None)),
                SchemaFieldKind::Uuid => Ok(Self::Uuid(None)),
                SchemaFieldKind::Date => Ok(Self::Date(None)),
                SchemaFieldKind::Time => Ok(Self::Time(None)),
                SchemaFieldKind::DateTime => Ok(Self::DateTime(None)),
                SchemaFieldKind::Timestamp => Ok(Self::Timestamp(None)),
                SchemaFieldKind::Json => Ok(Self::Json(None)),
            },
            serde_json::Value::Bool(value) => match kind {
                SchemaFieldKind::Boolean => Ok(Self::Boolean(Some(*value))),
                SchemaFieldKind::Byte => Ok(Self::Byte(Some(vec![*value as u8]))),
                _ => return Err(Error::msg("wrong value type")),
            },
            serde_json::Value::Number(value) => match kind {
                SchemaFieldKind::TinyInteger => match value.as_i64() {
                    Some(value) => match i8::try_from(value) {
                        Ok(value) => Ok(Self::TinyInteger(Some(value))),
                        Err(err) => Err(err.into()),
                    },
                    None => Err(Error::msg("wrong value type")),
                },
                SchemaFieldKind::SmallInteger => match value.as_i64() {
                    Some(value) => match i16::try_from(value) {
                        Ok(value) => Ok(Self::SmallInteger(Some(value))),
                        Err(err) => Err(err.into()),
                    },
                    None => Err(Error::msg("wrong value type")),
                },
                SchemaFieldKind::Integer => match value.as_i64() {
                    Some(value) => match i32::try_from(value) {
                        Ok(value) => Ok(Self::Integer(Some(value))),
                        Err(err) => Err(err.into()),
                    },
                    None => Err(Error::msg("wrong value type")),
                },
                SchemaFieldKind::BigInteger => match value.as_i64() {
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
                SchemaFieldKind::String => Ok(Self::String(Some(value.to_string()))),
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
}
