use std::{any::Any, collections::hash_map::Keys, str::FromStr};

use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use anyhow::{Error, Result};
use bigdecimal::{num_traits::ToBytes, ToPrimitive};
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime, Utc};
use hb_db_scylladb::{
    db::ScyllaDb,
    model::collection::SchemaFieldPropsScyllaModel,
    query::record::{self, COUNT_TABLE},
};
use scylla::frame::{
    response::result::CqlValue,
    value::{Time, Timestamp, Value},
};
use uuid::Uuid;

use crate::{
    collection::{CollectionDao, SchemaFieldKind},
    Db,
};

pub struct RecordDao {
    table_name: String,
    data: HashMap<String, ColumnValue>,
}

impl RecordDao {
    pub fn new(table_name: &str, capacity: &Option<usize>) -> Self {
        let capacity = match capacity {
            Some(capacity) => capacity + 1,
            None => 1,
        };

        let mut data = HashMap::with_capacity(capacity);
        data.insert("_id".to_owned(), ColumnValue::Uuid(Some(Uuid::new_v4())));

        Self {
            table_name: table_name.to_owned(),
            data,
        }
    }

    pub fn new_table_name(collection_id: &Uuid) -> String {
        "record_".to_owned() + &collection_id.to_string().replace("-", "")
    }

    pub fn table_name(&self) -> &str {
        &self.table_name
    }

    pub fn data(&self) -> &HashMap<String, ColumnValue> {
        &self.data
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn get(&self, key: &str) -> Option<&ColumnValue> {
        self.data.get(key)
    }

    pub fn keys(&self) -> Keys<'_, String, ColumnValue> {
        self.data.keys()
    }

    pub fn insert(&mut self, key: &str, value: &ColumnValue) {
        self.data.insert(key.to_owned(), value.to_owned());
    }

    pub async fn db_create_table(db: &Db, collection: &CollectionDao) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => {
                Self::scylladb_create_table(
                    db,
                    collection.id(),
                    &collection
                        .schema_fields()
                        .iter()
                        .map(|(field_name, field_props)| {
                            (field_name.clone(), field_props.to_scylladb_model())
                        })
                        .collect::<HashMap<_, _>>(),
                )
                .await
            }
        }
    }

    pub async fn db_drop_table(db: &Db, collection_id: &Uuid) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => Self::scylladb_drop_table(db, collection_id).await,
        }
    }

    pub async fn db_check_table_existence(db: &Db, collection_id: &Uuid) -> Result<bool> {
        match db {
            Db::ScyllaDb(db) => Self::scylladb_check_table_existence(db, collection_id).await,
        }
    }

    pub async fn db_check_table_must_exist(db: &Db, collection_id: &Uuid) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => match Self::scylladb_check_table_existence(db, collection_id).await
            {
                Ok(is_exist) => match is_exist {
                    true => Ok(()),
                    false => Err(Error::msg(format!(
                        "Collection {collection_id} does not exist"
                    ))),
                },
                Err(err) => Err(err),
            },
        }
    }

    pub async fn db_add_columns(
        db: &Db,
        collection_id: &Uuid,
        columns: &HashMap<String, SchemaFieldPropsScyllaModel>,
    ) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => Self::scylladb_add_columns(db, collection_id, columns).await,
        }
    }

    pub async fn db_drop_columns(
        db: &Db,
        collection_id: &Uuid,
        columns: &HashSet<String>,
    ) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => Self::scylladb_drop_columns(db, collection_id, columns).await,
        }
    }

    pub async fn db_create_index(db: &Db, collection_id: &Uuid, index: &str) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => Self::scylladb_create_index(db, collection_id, index).await,
        }
    }

    pub async fn db_drop_index(db: &Db, collection_id: &Uuid, index: &str) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => Self::scylladb_drop_index(db, collection_id, index).await,
        }
    }

    pub async fn db_insert(&self, db: &Db) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => Self::scylladb_insert(self, db).await,
        }
    }

    pub async fn db_select(db: &Db, collection_data: &CollectionDao, id: &Uuid) -> Result<Self> {
        match db {
            Db::ScyllaDb(db) => {
                let table_name = Self::new_table_name(collection_data.id());
                let mut columns = Vec::with_capacity(collection_data.schema_fields().len());
                let mut columns_props = Vec::with_capacity(collection_data.schema_fields().len());
                for (column, props) in collection_data.schema_fields() {
                    columns.push(column.to_owned());
                    columns_props.push(*props)
                }
                let scylladb_data = Self::scylladb_select(db, &table_name, &columns, id).await?;
                let mut data = HashMap::with_capacity(scylladb_data.len());
                for (idx, value) in scylladb_data.iter().enumerate() {
                    match value {
                        Some(value) => {
                            match ColumnValue::from_scylladb_model(columns_props[idx].kind(), value)
                            {
                                Ok(value) => data.insert(columns[idx].to_owned(), value),
                                Err(err) => return Err(err.into()),
                            }
                        }
                        None => data.insert(
                            columns[idx].to_owned(),
                            ColumnValue::none(columns_props[idx].kind()),
                        ),
                    };
                }
                Ok(Self { table_name, data })
            }
        }
    }

    pub async fn db_delete(db: &Db, collection_id: &Uuid, id: &Uuid) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => {
                Self::scylladb_delete(db, &Self::new_table_name(collection_id), id).await
            }
        }
    }

    async fn scylladb_create_table(
        db: &ScyllaDb,
        collection_id: &Uuid,
        schema_fields: &HashMap<String, SchemaFieldPropsScyllaModel>,
    ) -> Result<()> {
        db.session_query(
            record::create_table(&Self::new_table_name(collection_id), schema_fields).as_str(),
            &[],
        )
        .await?;
        Ok(())
    }

    async fn scylladb_drop_table(db: &ScyllaDb, collection_id: &Uuid) -> Result<()> {
        db.session_query(
            record::drop_table(&RecordDao::new_table_name(collection_id)).as_str(),
            &[],
        )
        .await?;
        Ok(())
    }

    async fn scylladb_check_table_existence(db: &ScyllaDb, collection_id: &Uuid) -> Result<bool> {
        Ok(db
            .session_query(
                COUNT_TABLE,
                [&RecordDao::new_table_name(collection_id)].as_ref(),
            )
            .await?
            .first_row_typed::<(i64,)>()?
            .0
            > 0)
    }

    async fn scylladb_add_columns(
        db: &ScyllaDb,
        collection_id: &Uuid,
        columns: &HashMap<String, SchemaFieldPropsScyllaModel>,
    ) -> Result<()> {
        db.session_query(
            &record::add_columns(&Self::new_table_name(collection_id), columns),
            &[],
        )
        .await?;
        Ok(())
    }

    async fn scylladb_drop_columns(
        db: &ScyllaDb,
        collection_id: &Uuid,
        columns: &HashSet<String>,
    ) -> Result<()> {
        db.session_query(
            &record::drop_columns(&Self::new_table_name(collection_id), columns),
            &[],
        )
        .await?;
        Ok(())
    }

    async fn scylladb_create_index(db: &ScyllaDb, collection_id: &Uuid, index: &str) -> Result<()> {
        db.session_query(
            record::create_index(&Self::new_table_name(collection_id), index).as_str(),
            &[],
        )
        .await?;
        Ok(())
    }

    async fn scylladb_drop_index(db: &ScyllaDb, collection_id: &Uuid, index: &str) -> Result<()> {
        db.session_query(
            &record::drop_index(&Self::new_table_name(collection_id), index),
            &[],
        )
        .await?;
        Ok(())
    }

    async fn scylladb_insert(&self, db: &ScyllaDb) -> Result<()> {
        let mut cols = Vec::with_capacity(self.data.len());
        let mut vals = Vec::with_capacity(self.data.len());
        for (col, val) in &self.data {
            cols.push(col.to_owned());
            vals.push(val.to_scylladb_model());
        }
        db.execute(
            &record::insert(&self.table_name, &cols),
            vals.as_ref() as &[Box<dyn Value>],
        )
        .await?;
        Ok(())
    }

    async fn scylladb_select(
        db: &ScyllaDb,
        table_name: &str,
        columns: &Vec<String>,
        id: &Uuid,
    ) -> Result<Vec<Option<CqlValue>>> {
        let values = db
            .execute(&record::select(table_name, columns), [id].as_ref())
            .await?
            .first_row()?
            .columns;
        Ok(values)
    }

    async fn scylladb_delete(db: &ScyllaDb, table_name: &str, id: &Uuid) -> Result<()> {
        let mut column = HashSet::<String>::with_capacity(1);
        column.insert("_id".to_owned());
        db.execute(&record::delete(table_name, &column), [id].as_ref())
            .await?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum ColumnValue {
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

impl ColumnValue {
    pub fn from_serde_json(kind: &SchemaFieldKind, value: &serde_json::Value) -> Result<Self> {
        match value {
            serde_json::Value::Null => Ok(Self::none(kind)),
            serde_json::Value::Bool(value) => match kind {
                SchemaFieldKind::Bool => Ok(Self::Boolean(Some(*value))),
                SchemaFieldKind::Byte => Ok(Self::Byte(Some(vec![(*value).into()]))),
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
                SchemaFieldKind::Uuid => match Uuid::from_str(value) {
                    Ok(uuid) => Ok(Self::Uuid(Some(uuid))),
                    Err(err) => Err(err.into()),
                },
                SchemaFieldKind::Date => match NaiveDate::parse_from_str(value, "%Y-%m-%d") {
                    Ok(date) => Ok(Self::Date(Some(date))),
                    Err(err) => Err(err.into()),
                },
                SchemaFieldKind::Time => match NaiveTime::parse_from_str(value, "%H:%M:%S") {
                    Ok(time) => Ok(Self::Time(Some(time))),
                    Err(err) => Err(err.into()),
                },
                SchemaFieldKind::DateTime => match DateTime::parse_from_rfc3339(value) {
                    Ok(datetime) => Ok(Self::DateTime(Some(datetime))),
                    Err(err) => Err(err.into()),
                },
                SchemaFieldKind::Timestamp => match DateTime::parse_from_rfc3339(value) {
                    Ok(timestamp) => Ok(Self::Timestamp(Some(timestamp))),
                    Err(err) => Err(err.into()),
                },
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

    pub fn into_inner_boxed(&self) -> Box<dyn Any> {
        match self {
            ColumnValue::Boolean(data) => Box::new(*data),
            ColumnValue::TinyInteger(data) => Box::new(*data),
            ColumnValue::SmallInteger(data) => Box::new(*data),
            ColumnValue::Integer(data) => Box::new(*data),
            ColumnValue::BigInteger(data) => Box::new(*data),
            ColumnValue::Float(data) => Box::new(*data),
            ColumnValue::Double(data) => Box::new(*data),
            ColumnValue::String(data) => Box::new(data.to_owned()),
            ColumnValue::Byte(data) => Box::new(data.to_owned()),
            ColumnValue::Uuid(data) => Box::new(*data),
            ColumnValue::Date(data) => Box::new(*data),
            ColumnValue::Time(data) => Box::new(*data),
            ColumnValue::DateTime(data) => Box::new(*data),
            ColumnValue::Timestamp(data) => Box::new(*data),
            ColumnValue::Json(data) => Box::new(data.to_owned()),
        }
    }

    pub fn from_scylladb_model(kind: &SchemaFieldKind, value: &CqlValue) -> Result<Self> {
        match value {
            CqlValue::Ascii(value) => match kind {
                SchemaFieldKind::String => Ok(Self::String(Some(value.to_owned()))),
                SchemaFieldKind::Byte => Ok(Self::Byte(Some(value.as_bytes().to_vec()))),
                _ => return Err(Error::msg("wrong value type")),
            },
            CqlValue::Boolean(value) => match kind {
                SchemaFieldKind::Bool => Ok(Self::Boolean(Some(*value))),
                SchemaFieldKind::Byte => Ok(Self::Byte(Some([(*value).into()].to_vec()))),
                _ => return Err(Error::msg("wrong value type")),
            },
            CqlValue::Blob(value) => match kind {
                SchemaFieldKind::Byte => Ok(Self::Byte(Some(value.to_owned()))),
                _ => return Err(Error::msg("wrong value type")),
            },
            CqlValue::Counter(value) => match kind {
                SchemaFieldKind::BigInt => Ok(Self::BigInteger(Some(value.0))),
                SchemaFieldKind::Byte => Ok(Self::Byte(Some(value.0.to_be_bytes().to_vec()))),
                _ => return Err(Error::msg("wrong value type")),
            },
            CqlValue::Decimal(value) => match value.to_f64() {
                Some(value) => match kind {
                    SchemaFieldKind::Double => Ok(Self::Double(Some(value))),
                    SchemaFieldKind::Byte => Ok(Self::Byte(Some(value.to_be_bytes().to_vec()))),
                    _ => return Err(Error::msg("wrong value type")),
                },
                None => return Err(Error::msg("wrong value type")),
            },
            CqlValue::Date(value) => match kind {
                SchemaFieldKind::Date => Ok(Self::Date(
                    NaiveDate::from_ymd_opt(1970, 1, 1)
                        .unwrap()
                        .checked_add_signed(
                            chrono::Duration::days((*value).into())
                                - chrono::Duration::days(1 << 31),
                        ),
                )),
                SchemaFieldKind::Byte => Ok(Self::Byte(Some(value.to_be_bytes().to_vec()))),
                _ => return Err(Error::msg("wrong value type")),
            },
            CqlValue::Double(value) => match kind {
                SchemaFieldKind::Double => Ok(Self::Double(Some(*value))),
                SchemaFieldKind::Byte => Ok(Self::Byte(Some(value.to_be_bytes().to_vec()))),
                _ => return Err(Error::msg("wrong value type")),
            },
            CqlValue::Duration(value) => match kind {
                SchemaFieldKind::Date => Ok(Self::Date(
                    NaiveDate::from_ymd_opt(1970, 1, 1)
                        .unwrap()
                        .checked_add_signed(
                            chrono::Duration::days(value.days.into())
                                - chrono::Duration::days(1 << 31),
                        ),
                )),
                SchemaFieldKind::Time => {
                    let nano = match (chrono::Duration::nanoseconds(value.nanoseconds)
                        - chrono::Duration::days(1 << 31))
                    .num_nanoseconds()
                    {
                        Some(nano) => match u32::try_from(nano) {
                            Ok(nano) => nano,
                            Err(_) => return Err(Error::msg("wrong value type")),
                        },
                        None => return Err(Error::msg("wrong value type")),
                    };
                    match NaiveTime::from_hms_nano_opt(0, 0, 0, nano) {
                        Some(value) => Ok(Self::Time(Some(value))),
                        None => return Err(Error::msg("wrong value type")),
                    }
                }
                SchemaFieldKind::DateTime => Ok(Self::DateTime(Some(todo!()))),
                SchemaFieldKind::Timestamp => Ok(Self::Timestamp(Some(todo!()))),
                SchemaFieldKind::Byte => {
                    Ok(Self::Byte(Some(value.nanoseconds.to_be_bytes().to_vec())))
                }
                _ => return Err(Error::msg("wrong value type")),
            },
            CqlValue::Empty => match kind {
                SchemaFieldKind::Bool => Ok(Self::Boolean(None)),
                SchemaFieldKind::TinyInt => Ok(Self::TinyInteger(None)),
                SchemaFieldKind::SmallInt => Ok(Self::SmallInteger(None)),
                SchemaFieldKind::Int => Ok(Self::Integer(None)),
                SchemaFieldKind::BigInt => Ok(Self::BigInteger(None)),
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
            CqlValue::Float(value) => match kind {
                SchemaFieldKind::Float => Ok(Self::Float(Some(*value))),
                SchemaFieldKind::Double => Ok(Self::Double(Some((*value).into()))),
                SchemaFieldKind::Byte => Ok(Self::Byte(Some(value.to_be_bytes().to_vec()))),
                _ => return Err(Error::msg("wrong value type")),
            },
            CqlValue::Int(value) => match kind {
                SchemaFieldKind::Int => Ok(Self::Integer(Some(*value))),
                SchemaFieldKind::BigInt => Ok(Self::BigInteger(Some((*value).into()))),
                SchemaFieldKind::Byte => Ok(Self::Byte(Some(value.to_be_bytes().to_vec()))),
                _ => return Err(Error::msg("wrong value type")),
            },
            CqlValue::BigInt(value) => match kind {
                SchemaFieldKind::BigInt => Ok(Self::BigInteger(Some(*value))),
                SchemaFieldKind::Byte => Ok(Self::Byte(Some(value.to_be_bytes().to_vec()))),
                _ => return Err(Error::msg("wrong value type")),
            },
            CqlValue::Text(value) => match kind {
                SchemaFieldKind::String => Ok(Self::String(Some(value.to_owned()))),
                SchemaFieldKind::Byte => Ok(Self::Byte(Some(value.as_bytes().to_vec()))),
                _ => return Err(Error::msg("wrong value type")),
            },
            CqlValue::Timestamp(value) => match kind {
                SchemaFieldKind::DateTime => Ok(Self::DateTime(Some(todo!()))),
                SchemaFieldKind::Timestamp => Ok(Self::Timestamp(Some(todo!()))),
                SchemaFieldKind::Byte => Ok(Self::Byte(Some(todo!()))),
                _ => return Err(Error::msg("wrong value type")),
            },
            CqlValue::Inet(value) => match kind {
                SchemaFieldKind::String => Ok(Self::String(Some(value.to_string()))),
                SchemaFieldKind::Byte => {
                    Ok(Self::Byte(Some(value.to_string().as_bytes().to_vec())))
                }
                _ => return Err(Error::msg("wrong value type")),
            },
            CqlValue::List(value) => match kind {
                SchemaFieldKind::Byte => Ok(Self::Byte(Some(todo!()))),
                SchemaFieldKind::Json => Ok(Self::Json(Some(todo!()))),
                _ => return Err(Error::msg("wrong value type")),
            },
            CqlValue::Map(value) => match kind {
                SchemaFieldKind::Byte => Ok(Self::Byte(Some(todo!()))),
                SchemaFieldKind::Json => Ok(Self::Json(Some(todo!()))),
                _ => return Err(Error::msg("wrong value type")),
            },
            CqlValue::Set(value) => match kind {
                SchemaFieldKind::Byte => Ok(Self::Byte(Some(todo!()))),
                _ => return Err(Error::msg("wrong value type")),
            },
            CqlValue::UserDefinedType {
                keyspace,
                type_name,
                fields,
            } => match kind {
                SchemaFieldKind::Byte => Ok(Self::Byte(Some(todo!()))),
                SchemaFieldKind::Json => Ok(Self::Json(Some(todo!()))),
                _ => return Err(Error::msg("wrong value type")),
            },
            CqlValue::SmallInt(value) => match kind {
                SchemaFieldKind::SmallInt => Ok(Self::SmallInteger(Some(*value))),
                SchemaFieldKind::Int => Ok(Self::Integer(Some((*value).into()))),
                SchemaFieldKind::BigInt => Ok(Self::BigInteger(Some((*value).into()))),
                SchemaFieldKind::Byte => Ok(Self::Byte(Some(value.to_be_bytes().to_vec()))),
                _ => return Err(Error::msg("wrong value type")),
            },
            CqlValue::TinyInt(value) => match kind {
                SchemaFieldKind::TinyInt => Ok(Self::TinyInteger(Some(*value))),
                SchemaFieldKind::SmallInt => Ok(Self::SmallInteger(Some((*value).into()))),
                SchemaFieldKind::Int => Ok(Self::Integer(Some((*value).into()))),
                SchemaFieldKind::BigInt => Ok(Self::BigInteger(Some((*value).into()))),
                SchemaFieldKind::Byte => Ok(Self::Byte(Some(value.to_be_bytes().to_vec()))),
                _ => return Err(Error::msg("wrong value type")),
            },
            CqlValue::Time(value) => match kind {
                SchemaFieldKind::Time => Ok(Self::Time(Some(todo!()))),
                SchemaFieldKind::Byte => Ok(Self::Byte(Some(todo!()))),
                _ => return Err(Error::msg("wrong value type")),
            },
            CqlValue::Timeuuid(value) => match kind {
                SchemaFieldKind::Byte => Ok(Self::Byte(Some(value.as_bytes().to_vec()))),
                SchemaFieldKind::Uuid => Ok(Self::Uuid(Some(*value))),
                _ => return Err(Error::msg("wrong value type")),
            },
            CqlValue::Tuple(value) => match kind {
                SchemaFieldKind::Byte => Ok(Self::Byte(Some(todo!()))),
                _ => return Err(Error::msg("wrong value type")),
            },
            CqlValue::Uuid(value) => match kind {
                SchemaFieldKind::Uuid => Ok(Self::Uuid(Some(*value))),
                SchemaFieldKind::Byte => Ok(Self::Byte(Some(value.as_bytes().to_vec()))),
                _ => return Err(Error::msg("wrong value type")),
            },
            CqlValue::Varint(value) => match value.to_i64() {
                Some(value) => match kind {
                    SchemaFieldKind::BigInt => Ok(Self::BigInteger(Some(value))),
                    SchemaFieldKind::Byte => Ok(Self::Byte(Some(value.to_be_bytes().to_vec()))),
                    _ => return Err(Error::msg("wrong value type")),
                },
                None => return Err(Error::msg("wrong value type")),
            },
        }
    }

    pub fn to_scylladb_model(&self) -> Box<dyn Value> {
        match self {
            ColumnValue::Boolean(data) => Box::new(*data),
            ColumnValue::TinyInteger(data) => Box::new(*data),
            ColumnValue::SmallInteger(data) => Box::new(*data),
            ColumnValue::Integer(data) => Box::new(*data),
            ColumnValue::BigInteger(data) => Box::new(*data),
            ColumnValue::Float(data) => Box::new(*data),
            ColumnValue::Double(data) => Box::new(*data),
            ColumnValue::String(data) => Box::new(data.to_owned()),
            ColumnValue::Byte(data) => Box::new(data.to_owned()),
            ColumnValue::Uuid(data) => Box::new(*data),
            ColumnValue::Date(data) => Box::new(*data),
            ColumnValue::Time(data) => Box::new(match data {
                Some(data) => Some(Time(*data - NaiveTime::MIN)),
                None => None,
            }),
            ColumnValue::DateTime(data) => Box::new(match data {
                Some(data) => Some(Timestamp(Utc::now().signed_duration_since(data))),
                None => None,
            }),
            ColumnValue::Timestamp(data) => Box::new(match data {
                Some(data) => Some(Timestamp(Utc::now().signed_duration_since(data))),
                None => None,
            }),
            ColumnValue::Json(data) => Box::new(data.to_owned()),
        }
    }
}
