use std::str::FromStr;

use anyhow::{Error, Result};
use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use hb_db_mysql::model::value::ColumnKind as ColumnKindMysql;
use hb_db_postgresql::model::value::ColumnKind as ColumnKindPostgres;
use hb_db_scylladb::model::value::ColumnKind as ColumnKindScylla;
use hb_db_sqlite::model::value::ColumnKind as ColumnKindSqlite;
use num_bigint::BigInt;
use scylla::{
    frame::response::result::CqlValue as ScyllaCqlValue,
    serialize::value::SerializeCql as ScyllaSerializeCql,
};
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;
use uuid::Uuid;

use crate::util::conversion;

#[derive(Deserialize, Serialize, EnumIter, PartialEq, Clone, Copy)]
pub enum ColumnKind {
    Boolean,   // boolean
    TinyInt,   // 8-bit signed int
    SmallInt,  // 16-bit signed int
    Int,       // 32-bit signed int
    BigInt,    // 64-bit signed long
    Varint,    // Arbitrary-precision integer
    Float,     // 32-bit IEEE-754 floating point
    Double,    // 64-bit IEEE-754 floating point
    Decimal,   // Variable-precision decimal
    String,    // UTF8 encoded string
    Binary,    // Arbitrary bytes
    Uuid,      // A UUID (of any version)
    Date,      // A date (with no corresponding time value)
    Time,      // A time (with no corresponding date value)
    Timestamp, // A timestamp (date and time)
    Json,      // A json data format
}

impl ColumnKind {
    pub fn to_str(&self) -> &str {
        match self {
            Self::Boolean => "boolean",
            Self::TinyInt => "tinyint",
            Self::SmallInt => "smallint",
            Self::Int => "int",
            Self::BigInt => "bigint",
            Self::Varint => "varint",
            Self::Float => "float",
            Self::Double => "double",
            Self::Decimal => "decimal",
            Self::String => "string",
            Self::Binary => "binary",
            Self::Uuid => "uuid",
            Self::Date => "date",
            Self::Time => "time",
            Self::Timestamp => "timestamp",
            Self::Json => "json",
        }
    }

    pub fn from_str(str: &str) -> Result<Self> {
        match str {
            "boolean" => Ok(Self::Boolean),
            "tinyint" => Ok(Self::TinyInt),
            "smallint" => Ok(Self::SmallInt),
            "int" => Ok(Self::Int),
            "bigint" => Ok(Self::BigInt),
            "varint" => Ok(Self::Varint),
            "float" => Ok(Self::Float),
            "double" => Ok(Self::Double),
            "decimal" => Ok(Self::Decimal),
            "string" => Ok(Self::String),
            "binary" => Ok(Self::Binary),
            "uuid" => Ok(Self::Uuid),
            "date" => Ok(Self::Date),
            "time" => Ok(Self::Time),
            "timestamp" => Ok(Self::Timestamp),
            "json" => Ok(Self::Json),
            _ => Err(Error::msg(format!("Unknown schema field kind '{str}'"))),
        }
    }

    pub fn to_scylladb_model(&self) -> ColumnKindScylla {
        match self {
            Self::Boolean => ColumnKindScylla::Boolean,
            Self::TinyInt => ColumnKindScylla::TinyInt,
            Self::SmallInt => ColumnKindScylla::SmallInt,
            Self::Int => ColumnKindScylla::Int,
            Self::BigInt => ColumnKindScylla::BigInt,
            Self::Varint => ColumnKindScylla::Varint,
            Self::Float => ColumnKindScylla::Float,
            Self::Double => ColumnKindScylla::Double,
            Self::Decimal => ColumnKindScylla::Decimal,
            Self::String => ColumnKindScylla::Text,
            Self::Binary | Self::Json => ColumnKindScylla::Blob,
            Self::Uuid => ColumnKindScylla::Uuid,
            Self::Date => ColumnKindScylla::Date,
            Self::Time => ColumnKindScylla::Time,
            Self::Timestamp => ColumnKindScylla::Timestamp,
        }
    }

    pub fn to_postgresdb_model(&self) -> ColumnKindPostgres {
        match self {
            Self::Boolean => ColumnKindPostgres::Bool,
            Self::TinyInt => ColumnKindPostgres::Char,
            Self::SmallInt => ColumnKindPostgres::Smallint,
            Self::Int => ColumnKindPostgres::Integer,
            Self::BigInt => ColumnKindPostgres::Bigint,
            Self::Varint => ColumnKindPostgres::Numeric,
            Self::Float => ColumnKindPostgres::Real,
            Self::Double => ColumnKindPostgres::DoublePrecision,
            Self::Decimal => ColumnKindPostgres::Numeric,
            Self::String => ColumnKindPostgres::Varchar,
            Self::Binary => ColumnKindPostgres::Bytea,
            Self::Uuid => ColumnKindPostgres::Uuid,
            Self::Date => ColumnKindPostgres::Date,
            Self::Time => ColumnKindPostgres::Time,
            Self::Timestamp => ColumnKindPostgres::Timestamptz6,
            Self::Json => ColumnKindPostgres::Jsonb,
        }
    }

    pub fn to_mysqldb_model(&self) -> ColumnKindMysql {
        match self {
            Self::Boolean => ColumnKindMysql::Bool,
            Self::TinyInt => ColumnKindMysql::Tinyint,
            Self::SmallInt => ColumnKindMysql::Smallint,
            Self::Int => ColumnKindMysql::Int,
            Self::BigInt => ColumnKindMysql::Bigint,
            Self::Binary | Self::Varint | Self::Decimal => ColumnKindMysql::Blob,
            Self::Float => ColumnKindMysql::Float,
            Self::Double => ColumnKindMysql::Double,
            Self::String => ColumnKindMysql::Varchar255,
            Self::Uuid => ColumnKindMysql::Binary16,
            Self::Date => ColumnKindMysql::Date,
            Self::Time => ColumnKindMysql::Time,
            Self::Timestamp => ColumnKindMysql::Timestamp6,
            Self::Json => ColumnKindMysql::Json,
        }
    }

    pub fn to_sqlitedb_model(&self) -> ColumnKindSqlite {
        match self {
            Self::Boolean => ColumnKindSqlite::Boolean,
            Self::TinyInt | Self::SmallInt | Self::Int => ColumnKindSqlite::Integer,
            Self::BigInt => ColumnKindSqlite::Bigint,
            Self::Binary | Self::Varint | Self::Decimal | Self::Uuid | Self::Json => {
                ColumnKindSqlite::Blob
            }
            Self::Float | Self::Double => ColumnKindSqlite::Real,
            Self::String => ColumnKindSqlite::Text,
            Self::Date => ColumnKindSqlite::Date,
            Self::Time => ColumnKindSqlite::Time,
            Self::Timestamp => ColumnKindSqlite::Timestamp,
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub enum ColumnValue {
    Boolean(Option<bool>),
    TinyInteger(Option<i8>),
    SmallInteger(Option<i16>),
    Integer(Option<i32>),
    BigInteger(Option<i64>),
    VarInteger(Option<BigInt>),
    Float(Option<f32>),
    Double(Option<f64>),
    Decimal(Option<BigDecimal>),
    String(Option<String>),
    Binary(Option<Vec<u8>>),
    Uuid(Option<Uuid>),
    Date(Option<NaiveDate>),
    Time(Option<NaiveTime>),
    Timestamp(Option<DateTime<Utc>>),
    Json(Option<String>),
}

impl ColumnValue {
    pub fn none(kind: &ColumnKind) -> Self {
        match kind {
            ColumnKind::Boolean => Self::Boolean(None),
            ColumnKind::TinyInt => Self::TinyInteger(None),
            ColumnKind::SmallInt => Self::SmallInteger(None),
            ColumnKind::Int => Self::Integer(None),
            ColumnKind::BigInt => Self::BigInteger(None),
            ColumnKind::Varint => Self::VarInteger(None),
            ColumnKind::Float => Self::Float(None),
            ColumnKind::Double => Self::Double(None),
            ColumnKind::Decimal => Self::Decimal(None),
            ColumnKind::String => Self::String(None),
            ColumnKind::Binary => Self::Binary(None),
            ColumnKind::Uuid => Self::Uuid(None),
            ColumnKind::Date => Self::Date(None),
            ColumnKind::Time => Self::Time(None),
            ColumnKind::Timestamp => Self::Timestamp(None),
            ColumnKind::Json => Self::Json(None),
        }
    }

    pub fn from_serde_json(kind: &ColumnKind, value: &serde_json::Value) -> Result<Self> {
        match value {
            serde_json::Value::Null => Ok(Self::none(kind)),
            serde_json::Value::Bool(value) => match kind {
                ColumnKind::Boolean => Ok(Self::Boolean(Some(*value))),
                ColumnKind::Binary => Ok(Self::Binary(Some(vec![(*value).into()]))),
                ColumnKind::Json => Ok(Self::Json(Some(value.to_string()))),
                _ => return Err(Error::msg("Wrong value type")),
            },
            serde_json::Value::Number(value) => match kind {
                ColumnKind::TinyInt => match value.as_i64() {
                    Some(value) => match i8::try_from(value) {
                        Ok(value) => Ok(Self::TinyInteger(Some(value))),
                        Err(err) => Err(err.into()),
                    },
                    None => Err(Error::msg("Wrong value type")),
                },
                ColumnKind::SmallInt => match value.as_i64() {
                    Some(value) => match i16::try_from(value) {
                        Ok(value) => Ok(Self::SmallInteger(Some(value))),
                        Err(err) => Err(err.into()),
                    },
                    None => Err(Error::msg("Wrong value type")),
                },
                ColumnKind::Int => match value.as_i64() {
                    Some(value) => match i32::try_from(value) {
                        Ok(value) => Ok(Self::Integer(Some(value))),
                        Err(err) => Err(err.into()),
                    },
                    None => Err(Error::msg("Wrong value type")),
                },
                ColumnKind::BigInt => match value.as_i64() {
                    Some(value) => Ok(Self::BigInteger(Some(value))),
                    None => Err(Error::msg("Wrong value type")),
                },
                ColumnKind::Float => match value.as_f64() {
                    Some(value) => {
                        let value = value as f32;
                        if value.is_finite() {
                            Ok(Self::Float(Some(value)))
                        } else {
                            Err(Error::msg("Wrong value type"))
                        }
                    }
                    None => Err(Error::msg("Wrong value type")),
                },
                ColumnKind::Double => match value.as_f64() {
                    Some(value) => Ok(Self::Double(Some(value))),
                    None => Err(Error::msg("Wrong value type")),
                },
                ColumnKind::Binary => Ok(Self::Binary(Some(value.to_string().into_bytes()))),
                ColumnKind::Json => Ok(Self::Json(Some(value.to_string()))),
                _ => return Err(Error::msg("Wrong value type")),
            },
            serde_json::Value::String(value) => match kind {
                ColumnKind::Varint => Ok(Self::VarInteger(Some(BigInt::from_str(
                    &value.to_string(),
                )?))),
                ColumnKind::Decimal => Ok(Self::Decimal(Some(BigDecimal::from_str(
                    &value.to_string(),
                )?))),
                ColumnKind::String => Ok(Self::String(Some(value.to_owned()))),
                ColumnKind::Binary => Ok(Self::Binary(Some(value.as_bytes().to_vec()))),
                ColumnKind::Uuid => match Uuid::parse_str(value) {
                    Ok(uuid) => Ok(Self::Uuid(Some(uuid))),
                    Err(err) => Err(err.into()),
                },
                ColumnKind::Date => match NaiveDate::parse_from_str(value, "%Y-%m-%d") {
                    Ok(date) => Ok(Self::Date(Some(date))),
                    Err(err) => Err(err.into()),
                },
                ColumnKind::Time => match NaiveTime::parse_from_str(value, "%H:%M:%S%.f") {
                    Ok(time) => Ok(Self::Time(Some(time))),
                    Err(err) => Err(err.into()),
                },
                ColumnKind::Timestamp => match DateTime::parse_from_rfc3339(value) {
                    Ok(timestamp) => Ok(Self::Timestamp(Some(timestamp.with_timezone(&Utc)))),
                    Err(err) => Err(err.into()),
                },
                ColumnKind::Json => Ok(Self::Json(Some(value.to_owned()))),
                _ => return Err(Error::msg("Wrong value type")),
            },
            serde_json::Value::Array(value) => match kind {
                ColumnKind::Binary => {
                    let mut bytes = Vec::with_capacity(value.len());
                    for value in value.iter() {
                        match value {
                            serde_json::Value::Number(number) => {
                                if let Some(number) = number.as_u64() {
                                    if let Ok(number) = u8::try_from(number) {
                                        bytes.append(&mut vec![number]);
                                        continue;
                                    }
                                }
                                return Err(Error::msg("Wrong value type"));
                            }
                            serde_json::Value::String(string) => {
                                bytes.append(&mut string.as_bytes().to_vec())
                            }
                            _ => return Err(Error::msg("Wrong value type")),
                        }
                    }
                    Ok(Self::Binary(Some(bytes)))
                }
                ColumnKind::Json => Ok(Self::Json(Some(serde_json::json!(value).to_string()))),
                _ => return Err(Error::msg("Wrong value type")),
            },
            serde_json::Value::Object(value) => match kind {
                ColumnKind::Binary => Ok(Self::Binary(Some(
                    serde_json::json!(value).to_string().into_bytes(),
                ))),
                ColumnKind::Json => Ok(Self::Json(Some(serde_json::json!(value).to_string()))),
                _ => return Err(Error::msg("Wrong value type")),
            },
        }
    }

    pub fn to_serde_json(&self) -> Result<serde_json::Value> {
        match self {
            Self::Boolean(data) => match data {
                Some(data) => Ok(serde_json::json!(data)),
                None => Ok(serde_json::Value::Null),
            },
            Self::TinyInteger(data) => match data {
                Some(data) => Ok(serde_json::json!(data)),
                None => Ok(serde_json::Value::Null),
            },
            Self::SmallInteger(data) => match data {
                Some(data) => Ok(serde_json::json!(data)),
                None => Ok(serde_json::Value::Null),
            },
            Self::Integer(data) => match data {
                Some(data) => Ok(serde_json::json!(data)),
                None => Ok(serde_json::Value::Null),
            },
            Self::BigInteger(data) => match data {
                Some(data) => Ok(serde_json::json!(data)),
                None => Ok(serde_json::Value::Null),
            },
            Self::VarInteger(data) => match data {
                Some(data) => Ok(serde_json::json!(data.to_string())),
                None => Ok(serde_json::Value::Null),
            },
            Self::Float(data) => match data {
                Some(data) => Ok(serde_json::json!(data)),
                None => Ok(serde_json::Value::Null),
            },
            Self::Double(data) => match data {
                Some(data) => Ok(serde_json::json!(data)),
                None => Ok(serde_json::Value::Null),
            },
            Self::Decimal(data) => match data {
                Some(data) => Ok(serde_json::json!(data)),
                None => Ok(serde_json::Value::Null),
            },
            Self::String(data) => match data {
                Some(data) => Ok(serde_json::json!(data)),
                None => Ok(serde_json::Value::Null),
            },
            Self::Binary(data) => match data {
                Some(data) => Ok(serde_json::json!(data)),
                None => Ok(serde_json::Value::Null),
            },
            Self::Uuid(data) => match data {
                Some(data) => Ok(serde_json::json!(data)),
                None => Ok(serde_json::Value::Null),
            },
            Self::Date(data) => match data {
                Some(data) => Ok(serde_json::json!(data)),
                None => Ok(serde_json::Value::Null),
            },
            Self::Time(data) => match data {
                Some(data) => Ok(serde_json::json!(data)),
                None => Ok(serde_json::Value::Null),
            },
            Self::Timestamp(data) => match data {
                Some(data) => Ok(serde_json::json!(data)),
                None => Ok(serde_json::Value::Null),
            },
            Self::Json(data) => match data {
                Some(data) => match serde_json::from_str(data) {
                    Ok(data) => Ok(data),
                    Err(err) => Err(err.into()),
                },
                None => Ok(serde_json::Value::Null),
            },
        }
    }

    pub fn from_scylladb_model(kind: &ColumnKind, value: &ScyllaCqlValue) -> Result<Self> {
        match kind {
            ColumnKind::Boolean => Ok(Self::Boolean(value.as_boolean())),
            ColumnKind::TinyInt => Ok(Self::TinyInteger(value.as_tinyint())),
            ColumnKind::SmallInt => Ok(Self::SmallInteger(value.as_smallint())),
            ColumnKind::Int => Ok(Self::Integer(value.as_int())),
            ColumnKind::BigInt => Ok(Self::BigInteger(value.as_bigint())),
            ColumnKind::Varint => Ok(Self::VarInteger(match value.clone().into_cql_varint() {
                Some(value) => Some(BigInt::from_signed_bytes_be(
                    value.as_signed_bytes_be_slice(),
                )),
                None => None,
            })),
            ColumnKind::Float => Ok(Self::Float(value.as_float())),
            ColumnKind::Double => Ok(Self::Double(value.as_double())),
            ColumnKind::Decimal => Ok(Self::Decimal(match value.clone().into_cql_decimal() {
                Some(value) => Some({
                    let (digits, scale) = value.as_signed_be_bytes_slice_and_exponent();
                    let digits = BigInt::from_signed_bytes_be(digits);
                    BigDecimal::new(digits, scale.into())
                }),
                None => None,
            })),
            ColumnKind::String => Ok(Self::String(match value.as_text() {
                Some(value) => Some(value.to_owned()),
                None => None,
            })),
            ColumnKind::Binary => Ok(Self::Binary(match value.as_blob() {
                Some(value) => Some(value.to_vec()),
                None => None,
            })),
            ColumnKind::Uuid => Ok(Self::Uuid(value.as_uuid())),
            ColumnKind::Date => Ok(Self::Date(match value.as_cql_date() {
                Some(value) => Some(conversion::scylla_cql_date_to_naivedate(&value)?),
                None => None,
            })),
            ColumnKind::Time => Ok(Self::Time(match value.as_cql_time() {
                Some(value) => Some(conversion::scylla_cql_time_to_naivetime(&value)?),
                None => None,
            })),
            ColumnKind::Timestamp => Ok(Self::Timestamp(match value.as_cql_timestamp() {
                Some(value) => Some(conversion::scylla_cql_timestamp_to_datetime_utc(&value)?),
                None => None,
            })),
            ColumnKind::Json => Ok(Self::Json(match value.as_blob() {
                Some(value) => Some(std::str::from_utf8(&value)?.to_owned()),
                None => None,
            })),
        }
    }

    pub fn to_scylladb_model(&self) -> Result<Box<dyn ScyllaSerializeCql + Send + Sync>> {
        match self {
            Self::Boolean(data) => Ok(Box::new(*data)),
            Self::TinyInteger(data) => Ok(Box::new(*data)),
            Self::SmallInteger(data) => Ok(Box::new(*data)),
            Self::Integer(data) => Ok(Box::new(*data)),
            Self::BigInteger(data) => Ok(Box::new(*data)),
            Self::VarInteger(data) => Ok(Box::new(match data {
                Some(data) => Some(BigInt::from_signed_bytes_be(&data.to_signed_bytes_be())),
                None => None,
            })),
            Self::Float(data) => Ok(Box::new(*data)),
            Self::Double(data) => Ok(Box::new(*data)),
            Self::Decimal(data) => Ok(Box::new(match data {
                Some(data) => Some(BigDecimal::from_str(&data.to_string())?),
                None => None,
            })),
            Self::String(data) => Ok(Box::new(data.to_owned())),
            Self::Binary(data) => Ok(Box::new(data.to_owned())),
            Self::Uuid(data) => Ok(Box::new(*data)),
            Self::Date(data) => Ok(Box::new(match data {
                Some(data) => Some(conversion::naivedate_to_scylla_cql_date(data)?),
                None => None,
            })),
            Self::Time(data) => Ok(Box::new(match data {
                Some(data) => Some(conversion::naivetime_to_scylla_cql_time(data)?),
                None => None,
            })),
            Self::Timestamp(data) => Ok(Box::new(match data {
                Some(data) => Some(conversion::datetime_utc_to_scylla_cql_timestamp(data)),
                None => None,
            })),
            Self::Json(data) => Ok(Box::new(match data {
                Some(data) => Some(data.to_owned().into_bytes()),
                None => None,
            })),
        }
    }

    pub fn from_postgresdb_model(
        kind: &ColumnKind,
        index: &str,
        value: &sqlx::postgres::PgRow,
    ) -> Result<Self> {
        match kind {
            ColumnKind::Boolean => Ok(Self::Boolean(sqlx::Row::try_get(value, index)?)),
            ColumnKind::TinyInt => Ok(Self::TinyInteger(sqlx::Row::try_get(value, index)?)),
            ColumnKind::SmallInt => Ok(Self::SmallInteger(sqlx::Row::try_get(value, index)?)),
            ColumnKind::Int => Ok(Self::Integer(sqlx::Row::try_get(value, index)?)),
            ColumnKind::BigInt => Ok(Self::BigInteger(sqlx::Row::try_get(value, index)?)),
            ColumnKind::Varint => Ok(Self::VarInteger(
                match sqlx::Row::try_get::<Option<sqlx::types::BigDecimal>, _>(value, index)? {
                    Some(value) => Some(BigInt::from_str(&value.to_string())?),
                    None => None,
                },
            )),
            ColumnKind::Float => Ok(Self::Float(sqlx::Row::try_get(value, index)?)),
            ColumnKind::Double => Ok(Self::Double(sqlx::Row::try_get(value, index)?)),
            ColumnKind::Decimal => Ok(Self::Decimal(
                match sqlx::Row::try_get::<Option<sqlx::types::BigDecimal>, _>(value, index)? {
                    Some(value) => Some(BigDecimal::from_str(&value.to_string())?),
                    None => None,
                },
            )),
            ColumnKind::String => Ok(Self::String(sqlx::Row::try_get(value, index)?)),
            ColumnKind::Binary => Ok(Self::Binary(sqlx::Row::try_get(value, index)?)),
            ColumnKind::Uuid => Ok(Self::Uuid(sqlx::Row::try_get(value, index)?)),
            ColumnKind::Date => Ok(Self::Date(sqlx::Row::try_get(value, index)?)),
            ColumnKind::Time => Ok(Self::Time(sqlx::Row::try_get(value, index)?)),
            ColumnKind::Timestamp => Ok(Self::Timestamp(sqlx::Row::try_get(value, index)?)),
            ColumnKind::Json => Ok(Self::Json(
                match sqlx::Row::try_get::<Option<sqlx::types::Json<Vec<u8>>>, _>(value, index)? {
                    Some(value) => Some(std::str::from_utf8(&value.0)?.to_string()),
                    None => None,
                },
            )),
        }
    }

    pub fn to_postgresdb_model<'a>(
        &self,
        query: sqlx::query::Query<'a, sqlx::Postgres, sqlx::postgres::PgArguments>,
    ) -> Result<sqlx::query::Query<'a, sqlx::Postgres, sqlx::postgres::PgArguments>> {
        match self {
            Self::Boolean(data) => Ok(query.bind(*data)),
            Self::TinyInteger(data) => Ok(query.bind(*data)),
            Self::SmallInteger(data) => Ok(query.bind(*data)),
            Self::Integer(data) => Ok(query.bind(*data)),
            Self::BigInteger(data) => Ok(query.bind(*data)),
            Self::VarInteger(data) => Ok(query.bind(match data {
                Some(data) => Some(sqlx::types::BigDecimal::from_str(&data.to_string())?),
                None => None,
            })),
            Self::Float(data) => Ok(query.bind(*data)),
            Self::Double(data) => Ok(query.bind(*data)),
            Self::Decimal(data) => Ok(query.bind(match data {
                Some(data) => Some(sqlx::types::BigDecimal::from_str(&data.to_string())?),
                None => None,
            })),
            Self::String(data) => Ok(query.bind(data.to_owned())),
            Self::Binary(data) => Ok(query.bind(data.to_owned())),
            Self::Uuid(data) => Ok(query.bind(*data)),
            Self::Date(data) => Ok(query.bind(*data)),
            Self::Time(data) => Ok(query.bind(*data)),
            Self::Timestamp(data) => Ok(query.bind(*data)),
            Self::Json(data) => Ok(query.bind(match data {
                Some(data) => Some(sqlx::types::Json(data.to_owned().into_bytes())),
                None => None,
            })),
        }
    }

    pub fn to_postgresdb_model_as<'a, T>(
        &self,
        query: sqlx::query::QueryAs<'a, sqlx::Postgres, T, sqlx::postgres::PgArguments>,
    ) -> Result<sqlx::query::QueryAs<'a, sqlx::Postgres, T, sqlx::postgres::PgArguments>> {
        match self {
            Self::Boolean(data) => Ok(query.bind(*data)),
            Self::TinyInteger(data) => Ok(query.bind(*data)),
            Self::SmallInteger(data) => Ok(query.bind(*data)),
            Self::Integer(data) => Ok(query.bind(*data)),
            Self::BigInteger(data) => Ok(query.bind(*data)),
            Self::VarInteger(data) => Ok(query.bind(match data {
                Some(data) => Some(sqlx::types::BigDecimal::from_str(&data.to_string())?),
                None => None,
            })),
            Self::Float(data) => Ok(query.bind(*data)),
            Self::Double(data) => Ok(query.bind(*data)),
            Self::Decimal(data) => Ok(query.bind(match data {
                Some(data) => Some(sqlx::types::BigDecimal::from_str(&data.to_string())?),
                None => None,
            })),
            Self::String(data) => Ok(query.bind(data.to_owned())),
            Self::Binary(data) => Ok(query.bind(data.to_owned())),
            Self::Uuid(data) => Ok(query.bind(*data)),
            Self::Date(data) => Ok(query.bind(*data)),
            Self::Time(data) => Ok(query.bind(*data)),
            Self::Timestamp(data) => Ok(query.bind(*data)),
            Self::Json(data) => Ok(query.bind(match data {
                Some(data) => Some(sqlx::types::Json(data.to_owned().into_bytes())),
                None => None,
            })),
        }
    }

    pub fn from_mysqldb_model(
        kind: &ColumnKind,
        index: &str,
        value: &sqlx::mysql::MySqlRow,
    ) -> Result<Self> {
        match kind {
            ColumnKind::Boolean => Ok(Self::Boolean(sqlx::Row::try_get(value, index)?)),
            ColumnKind::TinyInt => Ok(Self::TinyInteger(sqlx::Row::try_get(value, index)?)),
            ColumnKind::SmallInt => Ok(Self::SmallInteger(sqlx::Row::try_get(value, index)?)),
            ColumnKind::Int => Ok(Self::Integer(sqlx::Row::try_get(value, index)?)),
            ColumnKind::BigInt => Ok(Self::BigInteger(sqlx::Row::try_get(value, index)?)),
            ColumnKind::Varint => Ok(Self::VarInteger(
                match sqlx::Row::try_get::<Option<&[u8]>, _>(value, index)? {
                    Some(value) => Some(BigInt::from_signed_bytes_be(value)),
                    None => None,
                },
            )),
            ColumnKind::Float => Ok(Self::Float(sqlx::Row::try_get(value, index)?)),
            ColumnKind::Double => Ok(Self::Double(sqlx::Row::try_get(value, index)?)),
            ColumnKind::Decimal => Ok(Self::Decimal(
                match sqlx::Row::try_get::<Option<&[u8]>, _>(value, index)? {
                    Some(value) => Some(BigDecimal::from_str(std::str::from_utf8(value)?)?),
                    None => None,
                },
            )),
            ColumnKind::String => Ok(Self::String(sqlx::Row::try_get(value, index)?)),
            ColumnKind::Binary => Ok(Self::Binary(sqlx::Row::try_get(value, index)?)),
            ColumnKind::Uuid => Ok(Self::Uuid(sqlx::Row::try_get(value, index)?)),
            ColumnKind::Date => Ok(Self::Date(sqlx::Row::try_get(value, index)?)),
            ColumnKind::Time => Ok(Self::Time(sqlx::Row::try_get(value, index)?)),
            ColumnKind::Timestamp => Ok(Self::Timestamp(
                sqlx::Row::try_get::<DateTime<Utc>, _>(value, index)?.into(),
            )),
            ColumnKind::Json => Ok(Self::Json(
                match sqlx::Row::try_get::<Option<sqlx::types::Json<Vec<u8>>>, _>(value, index)? {
                    Some(value) => Some(std::str::from_utf8(&value.0)?.to_owned()),
                    None => None,
                },
            )),
        }
    }

    pub fn to_mysqldb_model<'a>(
        &self,
        query: sqlx::query::Query<'a, sqlx::MySql, sqlx::mysql::MySqlArguments>,
    ) -> Result<sqlx::query::Query<'a, sqlx::MySql, sqlx::mysql::MySqlArguments>> {
        match self {
            Self::Boolean(data) => Ok(query.bind(*data)),
            Self::TinyInteger(data) => Ok(query.bind(*data)),
            Self::SmallInteger(data) => Ok(query.bind(*data)),
            Self::Integer(data) => Ok(query.bind(*data)),
            Self::BigInteger(data) => Ok(query.bind(*data)),
            Self::VarInteger(data) => Ok(query.bind(match data {
                Some(data) => Some(data.to_signed_bytes_be()),
                None => None,
            })),
            Self::Float(data) => Ok(query.bind(*data)),
            Self::Double(data) => Ok(query.bind(*data)),
            Self::Decimal(data) => Ok(query.bind(match data {
                Some(data) => Some(data.to_string().into_bytes()),
                None => None,
            })),
            Self::String(data) => Ok(query.bind(data.to_owned())),
            Self::Binary(data) => Ok(query.bind(data.to_owned())),
            Self::Uuid(data) => Ok(query.bind(*data)),
            Self::Date(data) => Ok(query.bind(*data)),
            Self::Time(data) => Ok(query.bind(*data)),
            Self::Timestamp(data) => Ok(query.bind(*data)),
            Self::Json(data) => Ok(query.bind(match data {
                Some(data) => Some(sqlx::types::Json(data.to_owned().into_bytes())),
                None => None,
            })),
        }
    }

    pub fn to_mysqldb_model_as<'a, T>(
        &self,
        query: sqlx::query::QueryAs<'a, sqlx::MySql, T, sqlx::mysql::MySqlArguments>,
    ) -> Result<sqlx::query::QueryAs<'a, sqlx::MySql, T, sqlx::mysql::MySqlArguments>> {
        match self {
            Self::Boolean(data) => Ok(query.bind(*data)),
            Self::TinyInteger(data) => Ok(query.bind(*data)),
            Self::SmallInteger(data) => Ok(query.bind(*data)),
            Self::Integer(data) => Ok(query.bind(*data)),
            Self::BigInteger(data) => Ok(query.bind(*data)),
            Self::VarInteger(data) => Ok(query.bind(match data {
                Some(data) => Some(data.to_signed_bytes_be()),
                None => None,
            })),
            Self::Float(data) => Ok(query.bind(*data)),
            Self::Double(data) => Ok(query.bind(*data)),
            Self::Decimal(data) => Ok(query.bind(match data {
                Some(data) => Some(data.to_string().into_bytes()),
                None => None,
            })),
            Self::String(data) => Ok(query.bind(data.to_owned())),
            Self::Binary(data) => Ok(query.bind(data.to_owned())),
            Self::Uuid(data) => Ok(query.bind(*data)),
            Self::Date(data) => Ok(query.bind(*data)),
            Self::Time(data) => Ok(query.bind(*data)),
            Self::Timestamp(data) => Ok(query.bind(*data)),
            Self::Json(data) => Ok(query.bind(match data {
                Some(data) => Some(sqlx::types::Json(data.to_owned().into_bytes())),
                None => None,
            })),
        }
    }

    pub fn from_sqlitedb_model(
        kind: &ColumnKind,
        index: &str,
        value: &sqlx::sqlite::SqliteRow,
    ) -> Result<Self> {
        match kind {
            ColumnKind::Boolean => Ok(Self::Boolean(sqlx::Row::try_get(value, index)?)),
            ColumnKind::TinyInt => Ok(Self::TinyInteger(sqlx::Row::try_get(value, index)?)),
            ColumnKind::SmallInt => Ok(Self::SmallInteger(sqlx::Row::try_get(value, index)?)),
            ColumnKind::Int => Ok(Self::Integer(sqlx::Row::try_get(value, index)?)),
            ColumnKind::BigInt => Ok(Self::BigInteger(sqlx::Row::try_get(value, index)?)),
            ColumnKind::Varint => Ok(Self::VarInteger(
                match sqlx::Row::try_get::<Option<&[u8]>, _>(value, index)? {
                    Some(value) => Some(BigInt::from_signed_bytes_be(value)),
                    None => None,
                },
            )),
            ColumnKind::Float => Ok(Self::Float(sqlx::Row::try_get(value, index)?)),
            ColumnKind::Double => Ok(Self::Double(sqlx::Row::try_get(value, index)?)),
            ColumnKind::Decimal => Ok(Self::Decimal(
                match sqlx::Row::try_get::<Option<&[u8]>, _>(value, index)? {
                    Some(value) => Some(BigDecimal::from_str(std::str::from_utf8(value)?)?),
                    None => None,
                },
            )),
            ColumnKind::String => Ok(Self::String(sqlx::Row::try_get(value, index)?)),
            ColumnKind::Binary => Ok(Self::Binary(sqlx::Row::try_get(value, index)?)),
            ColumnKind::Uuid => Ok(Self::Uuid(sqlx::Row::try_get(value, index)?)),
            ColumnKind::Date => Ok(Self::Date(sqlx::Row::try_get(value, index)?)),
            ColumnKind::Time => Ok(Self::Time(sqlx::Row::try_get(value, index)?)),
            ColumnKind::Timestamp => Ok(Self::Timestamp(sqlx::Row::try_get(value, index)?)),
            ColumnKind::Json => Ok(Self::Json(
                match sqlx::Row::try_get::<Option<Vec<u8>>, _>(value, index)? {
                    Some(value) => Some(std::str::from_utf8(&value)?.to_owned()),
                    None => None,
                },
            )),
        }
    }

    pub fn to_sqlitedb_model<'a>(
        &self,
        query: sqlx::query::Query<'a, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'a>>,
    ) -> Result<sqlx::query::Query<'a, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'a>>> {
        match self {
            Self::Boolean(data) => Ok(query.bind(*data)),
            Self::TinyInteger(data) => Ok(query.bind(*data)),
            Self::SmallInteger(data) => Ok(query.bind(*data)),
            Self::Integer(data) => Ok(query.bind(*data)),
            Self::BigInteger(data) => Ok(query.bind(*data)),
            Self::VarInteger(data) => Ok(query.bind(match data {
                Some(data) => Some(data.to_signed_bytes_be()),
                None => None,
            })),
            Self::Float(data) => Ok(query.bind(*data)),
            Self::Double(data) => Ok(query.bind(*data)),
            Self::Decimal(data) => Ok(query.bind(match data {
                Some(data) => Some(data.to_string().into_bytes()),
                None => None,
            })),
            Self::String(data) => Ok(query.bind(data.to_owned())),
            Self::Binary(data) => Ok(query.bind(data.to_owned())),
            Self::Uuid(data) => Ok(query.bind(*data)),
            Self::Date(data) => Ok(query.bind(*data)),
            Self::Time(data) => Ok(query.bind(*data)),
            Self::Timestamp(data) => Ok(query.bind(*data)),
            Self::Json(data) => Ok(query.bind(match data {
                Some(data) => Some(data.to_owned().into_bytes()),
                None => None,
            })),
        }
    }

    pub fn to_sqlitedb_model_as<'a, T>(
        &self,
        query: sqlx::query::QueryAs<'a, sqlx::Sqlite, T, sqlx::sqlite::SqliteArguments<'a>>,
    ) -> Result<sqlx::query::QueryAs<'a, sqlx::Sqlite, T, sqlx::sqlite::SqliteArguments<'a>>> {
        match self {
            Self::Boolean(data) => Ok(query.bind(*data)),
            Self::TinyInteger(data) => Ok(query.bind(*data)),
            Self::SmallInteger(data) => Ok(query.bind(*data)),
            Self::Integer(data) => Ok(query.bind(*data)),
            Self::BigInteger(data) => Ok(query.bind(*data)),
            Self::VarInteger(data) => Ok(query.bind(match data {
                Some(data) => Some(data.to_signed_bytes_be()),
                None => None,
            })),
            Self::Float(data) => Ok(query.bind(*data)),
            Self::Double(data) => Ok(query.bind(*data)),
            Self::Decimal(data) => Ok(query.bind(match data {
                Some(data) => Some(data.to_string().into_bytes()),
                None => None,
            })),
            Self::String(data) => Ok(query.bind(data.to_owned())),
            Self::Binary(data) => Ok(query.bind(data.to_owned())),
            Self::Uuid(data) => Ok(query.bind(*data)),
            Self::Date(data) => Ok(query.bind(*data)),
            Self::Time(data) => Ok(query.bind(*data)),
            Self::Timestamp(data) => Ok(query.bind(*data)),
            Self::Json(data) => Ok(query.bind(match data {
                Some(data) => Some(data.to_owned().into_bytes()),
                None => None,
            })),
        }
    }
}
