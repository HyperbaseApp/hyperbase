use std::{collections::hash_map::Keys, str::FromStr};

use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use anyhow::{Error, Result};
use bigdecimal::{num_traits::ToBytes, ToPrimitive};
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime};
use hb_db_scylladb::{
    db::ScyllaDb,
    model::{
        collection::SchemaFieldPropsScyllaModel,
        system::{COMPARISON_OPERATOR, LOGICAL_OPERATOR, ORDER_TYPE},
    },
    query::{record, system::COUNT_TABLE},
};
use scylla::frame::{
    response::result::CqlValue,
    value::{Time, Timestamp, Value},
};
use uuid::Uuid;

use crate::{
    collection::{CollectionDao, SchemaFieldKind, SchemaFieldPropsModel},
    Db,
};

pub struct RecordDao {
    table_name: String,
    data: HashMap<String, RecordColumnValue>,
}

impl RecordDao {
    pub fn new(collection_id: &Uuid, capacity: &Option<usize>) -> Self {
        let mut data = HashMap::with_capacity(match capacity {
            Some(capacity) => capacity + 1,
            None => 1,
        });
        data.insert(
            "_id".to_owned(),
            RecordColumnValue::Uuid(Some(Uuid::new_v4())),
        );

        Self {
            table_name: Self::new_table_name(collection_id),
            data,
        }
    }

    pub fn new_table_name(collection_id: &Uuid) -> String {
        "record_".to_owned() + &collection_id.to_string().replace("-", "")
    }

    pub fn table_name(&self) -> &str {
        &self.table_name
    }

    pub fn data(&self) -> &HashMap<String, RecordColumnValue> {
        &self.data
    }

    pub fn get(&self, key: &str) -> Option<&RecordColumnValue> {
        self.data.get(key)
    }

    pub fn keys(&self) -> Keys<'_, String, RecordColumnValue> {
        self.data.keys()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn upsert(&mut self, key: &str, value: &RecordColumnValue) {
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
                        "Collection '{collection_id}' doesn't exist"
                    ))),
                },
                Err(err) => Err(err),
            },
        }
    }

    pub async fn db_add_columns(
        db: &Db,
        collection_id: &Uuid,
        columns: &HashMap<String, SchemaFieldPropsModel>,
    ) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => {
                Self::scylladb_add_columns(
                    db,
                    collection_id,
                    &columns
                        .iter()
                        .map(|(col, col_props)| (col.to_owned(), col_props.to_scylladb_model()))
                        .collect(),
                )
                .await
            }
        }
    }

    pub async fn db_drop_columns(
        db: &Db,
        collection_id: &Uuid,
        column_names: &HashSet<String>,
    ) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => Self::scylladb_drop_columns(db, collection_id, column_names).await,
        }
    }

    pub async fn db_change_columns_type(
        db: &Db,
        collection_id: &Uuid,
        columns: &HashMap<String, SchemaFieldPropsModel>,
    ) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => {
                Self::scylladb_change_columns_type(
                    db,
                    collection_id,
                    &columns
                        .iter()
                        .map(|(col, col_props)| (col.to_owned(), col_props.to_scylladb_model()))
                        .collect(),
                )
                .await
            }
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
                let mut columns = Vec::with_capacity(collection_data.schema_fields().len() + 1);
                columns.push("_id".to_owned());
                let mut columns_props =
                    Vec::with_capacity(collection_data.schema_fields().len() + 1);
                columns_props.push(SchemaFieldPropsModel::new(&SchemaFieldKind::Uuid, &true));
                for (column, props) in collection_data.schema_fields() {
                    columns.push(column.to_owned());
                    columns_props.push(*props)
                }
                let scylladb_data = Self::scylladb_select(db, &table_name, &columns, id).await?;
                let mut data = HashMap::with_capacity(scylladb_data.len());
                for (idx, value) in scylladb_data.iter().enumerate() {
                    match value {
                        Some(value) => {
                            match RecordColumnValue::from_scylladb_model(
                                columns_props[idx].kind(),
                                value,
                            ) {
                                Ok(value) => data.insert(columns[idx].to_owned(), value),
                                Err(err) => return Err(err.into()),
                            }
                        }
                        None => data.insert(
                            columns[idx].to_owned(),
                            RecordColumnValue::none(columns_props[idx].kind()),
                        ),
                    };
                }
                Ok(Self { table_name, data })
            }
        }
    }

    pub async fn db_select_many(
        db: &Db,
        collection_data: &CollectionDao,
        filters: &RecordFilters,
        orders: &Vec<RecordOrder>,
        pagination: &RecordPagination,
    ) -> Result<(Vec<Self>, i64)> {
        match db {
            Db::ScyllaDb(db) => {
                let table_name = Self::new_table_name(collection_data.id());
                let mut columns = Vec::with_capacity(collection_data.schema_fields().len() + 1);
                columns.push("_id");
                let mut columns_props =
                    Vec::with_capacity(collection_data.schema_fields().len() + 1);
                columns_props.push(SchemaFieldPropsModel::new(&SchemaFieldKind::Uuid, &true));
                for (column, props) in collection_data.schema_fields() {
                    columns.push(column);
                    columns_props.push(*props)
                }
                let (scylladb_data_many, total) = Self::scylladb_select_many(
                    db,
                    &table_name,
                    &columns,
                    filters,
                    &orders,
                    pagination,
                )
                .await?;
                let mut data_many = Vec::with_capacity(scylladb_data_many.len());
                for scylladb_data in scylladb_data_many {
                    let mut data = HashMap::with_capacity(scylladb_data.len());
                    for (idx, value) in scylladb_data.iter().enumerate() {
                        match value {
                            Some(value) => match RecordColumnValue::from_scylladb_model(
                                columns_props[idx].kind(),
                                value,
                            ) {
                                Ok(value) => data.insert(columns[idx].to_owned(), value),
                                Err(err) => return Err(err.into()),
                            },
                            None => data.insert(
                                columns[idx].to_owned(),
                                RecordColumnValue::none(columns_props[idx].kind()),
                            ),
                        };
                    }
                    data_many.push(data);
                }
                Ok((
                    data_many
                        .iter()
                        .map(|data| Self {
                            table_name: table_name.to_owned(),
                            data: data.clone(),
                        })
                        .collect(),
                    total,
                ))
            }
        }
    }

    pub async fn db_update(&self, db: &Db) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => Self::scylladb_update(self, db).await,
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
        column_names: &HashSet<String>,
    ) -> Result<()> {
        db.session_query(
            &record::drop_columns(&Self::new_table_name(collection_id), column_names),
            &[],
        )
        .await?;
        Ok(())
    }

    async fn scylladb_change_columns_type(
        db: &ScyllaDb,
        collection_id: &Uuid,
        columns: &HashMap<String, SchemaFieldPropsScyllaModel>,
    ) -> Result<()> {
        db.session_query(
            &record::change_columns_type(&Self::new_table_name(collection_id), columns),
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
        let mut columns: Vec<_> = Vec::with_capacity(self.data.len());
        let mut values = Vec::with_capacity(self.data.len());
        for (col, val) in &self.data {
            columns.push(col.to_owned());
            values.push(val.to_scylladb_model());
        }
        db.execute(&record::insert(&self.table_name, &columns), &values)
            .await?;
        Ok(())
    }

    async fn scylladb_select(
        db: &ScyllaDb,
        table_name: &str,
        columns: &Vec<String>,
        id: &Uuid,
    ) -> Result<Vec<Option<CqlValue>>> {
        Ok(db
            .execute(&record::select(table_name, columns), [id].as_ref())
            .await?
            .first_row()?
            .columns)
    }

    async fn scylladb_select_many(
        db: &ScyllaDb,
        table_name: &str,
        columns: &Vec<&str>,
        filters: &RecordFilters,
        orders: &Vec<RecordOrder>,
        pagination: &RecordPagination,
    ) -> Result<(Vec<Vec<Option<CqlValue>>>, i64)> {
        let filter = filters.scylladb_filter_query(&None, 0)?;
        let mut order = HashMap::with_capacity(orders.len());
        for o in orders {
            if ORDER_TYPE.contains(&o.kind.as_str()) {
                order.insert(o.field.as_str(), o.kind.as_str());
            } else {
                return Err(Error::msg(format!(
                    "Order type '{}' is not supported",
                    &o.kind
                )));
            }
        }
        let mut values = filters.scylladb_values();
        let total_values = filters.scylladb_values();
        if let Some(limit) = pagination.limit() {
            values.push(Box::new(limit))
        }
        let query_select_many = record::select_many(
            table_name,
            columns,
            &filter,
            &order,
            &pagination.limit().is_some(),
        );
        let query_total = record::count(table_name, &filter);
        let (data, total) = tokio::try_join!(
            db.execute(&query_select_many, &values),
            db.execute(&query_total, &total_values)
        )?;
        Ok((
            data.rows()?
                .iter()
                .map(|row| row.columns.to_owned())
                .collect(),
            total.first_row_typed::<(i64,)>()?.0,
        ))
    }

    async fn scylladb_update(&self, db: &ScyllaDb) -> Result<()> {
        let mut columns = Vec::with_capacity(self.data.len());
        let mut values = Vec::with_capacity(self.data.len());
        for (col, val) in &self.data {
            if col != "_id" {
                columns.push(col.to_owned());
                values.push(val.to_scylladb_model());
            }
        }
        match self.data.get("_id") {
            Some(id) => values.push(id.to_scylladb_model()),
            None => return Err(Error::msg("Id is undefined")),
        }
        db.execute(&record::update(&self.table_name, &columns), &values)
            .await?;
        Ok(())
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
pub enum RecordColumnValue {
    Boolean(Option<bool>),
    TinyInteger(Option<i8>),
    SmallInteger(Option<i16>),
    Integer(Option<i32>),
    BigInteger(Option<i64>),
    Float(Option<f32>),
    Double(Option<f64>),
    String(Option<String>),
    Bytes(Option<Vec<u8>>),
    Uuid(Option<Uuid>),
    Date(Option<NaiveDate>),
    Time(Option<NaiveTime>),
    DateTime(Option<DateTime<FixedOffset>>),
    Timestamp(Option<DateTime<FixedOffset>>),
    Json(Option<String>),
}

impl RecordColumnValue {
    pub fn from_serde_json(kind: &SchemaFieldKind, value: &serde_json::Value) -> Result<Self> {
        match value {
            serde_json::Value::Null => Ok(Self::none(kind)),
            serde_json::Value::Bool(value) => match kind {
                SchemaFieldKind::Bool => Ok(Self::Boolean(Some(*value))),
                SchemaFieldKind::Bytes => Ok(Self::Bytes(Some(vec![(*value).into()]))),
                SchemaFieldKind::Json => Ok(Self::Json(Some(value.to_string()))),
                _ => return Err(Error::msg("Wrong value type")),
            },
            serde_json::Value::Number(value) => match kind {
                SchemaFieldKind::TinyInt => match value.as_i64() {
                    Some(value) => match i8::try_from(value) {
                        Ok(value) => Ok(Self::TinyInteger(Some(value))),
                        Err(err) => Err(err.into()),
                    },
                    None => Err(Error::msg("Wrong value type")),
                },
                SchemaFieldKind::SmallInt => match value.as_i64() {
                    Some(value) => match i16::try_from(value) {
                        Ok(value) => Ok(Self::SmallInteger(Some(value))),
                        Err(err) => Err(err.into()),
                    },
                    None => Err(Error::msg("Wrong value type")),
                },
                SchemaFieldKind::Int => match value.as_i64() {
                    Some(value) => match i32::try_from(value) {
                        Ok(value) => Ok(Self::Integer(Some(value))),
                        Err(err) => Err(err.into()),
                    },
                    None => Err(Error::msg("Wrong value type")),
                },
                SchemaFieldKind::BigInt => match value.as_i64() {
                    Some(value) => Ok(Self::BigInteger(Some(value))),
                    None => Err(Error::msg("Wrong value type")),
                },
                SchemaFieldKind::Float => match value.as_f64() {
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
                SchemaFieldKind::Double => match value.as_f64() {
                    Some(value) => Ok(Self::Double(Some(value))),
                    None => Err(Error::msg("Wrong value type")),
                },
                SchemaFieldKind::Bytes => Ok(Self::Bytes(Some(value.to_string().into_bytes()))),
                SchemaFieldKind::Json => Ok(Self::Json(Some(value.to_string()))),
                _ => return Err(Error::msg("Wrong value type")),
            },
            serde_json::Value::String(value) => match kind {
                SchemaFieldKind::String => Ok(Self::String(Some(value.to_owned()))),
                SchemaFieldKind::Bytes => Ok(Self::Bytes(Some(value.as_bytes().to_vec()))),
                SchemaFieldKind::Uuid => match Uuid::from_str(value) {
                    Ok(uuid) => Ok(Self::Uuid(Some(uuid))),
                    Err(err) => Err(err.into()),
                },
                SchemaFieldKind::Date => match NaiveDate::parse_from_str(value, "%Y-%m-%d") {
                    Ok(date) => Ok(Self::Date(Some(date))),
                    Err(err) => Err(err.into()),
                },
                SchemaFieldKind::Time => match NaiveTime::parse_from_str(value, "%H:%M:%S%.f") {
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
                SchemaFieldKind::Json => Ok(Self::Json(Some(serde_json::json!(value).to_string()))),
                _ => return Err(Error::msg("Wrong value type")),
            },
            serde_json::Value::Array(value) => match kind {
                SchemaFieldKind::Bytes => {
                    let mut bytes = Vec::with_capacity(value.len());
                    for value in value.iter() {
                        match value.as_str() {
                            Some(value) => bytes.append(&mut value.as_bytes().to_vec()),
                            None => return Err(Error::msg("Wrong value type")),
                        }
                    }
                    Ok(Self::Bytes(Some(bytes)))
                }
                SchemaFieldKind::Json => Ok(Self::Json(Some(serde_json::json!(value).to_string()))),
                _ => return Err(Error::msg("Wrong value type")),
            },
            serde_json::Value::Object(value) => match kind {
                SchemaFieldKind::Bytes => Ok(Self::Bytes(Some(
                    serde_json::json!(value).to_string().into_bytes(),
                ))),
                SchemaFieldKind::Json => Ok(Self::Json(Some(serde_json::json!(value).to_string()))),
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
            Self::Float(data) => match data {
                Some(data) => Ok(serde_json::json!(data)),
                None => Ok(serde_json::Value::Null),
            },
            Self::Double(data) => match data {
                Some(data) => Ok(serde_json::json!(data)),
                None => Ok(serde_json::Value::Null),
            },
            Self::String(data) => match data {
                Some(data) => Ok(serde_json::json!(data)),
                None => Ok(serde_json::Value::Null),
            },
            Self::Bytes(data) => match data {
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
            Self::DateTime(data) => match data {
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
            SchemaFieldKind::Bytes => Self::Bytes(None),
            SchemaFieldKind::Uuid => Self::Uuid(None),
            SchemaFieldKind::Date => Self::Date(None),
            SchemaFieldKind::Time => Self::Time(None),
            SchemaFieldKind::DateTime => Self::DateTime(None),
            SchemaFieldKind::Timestamp => Self::Timestamp(None),
            SchemaFieldKind::Json => Self::Json(None),
        }
    }

    pub fn from_scylladb_model(kind: &SchemaFieldKind, value: &CqlValue) -> Result<Self> {
        match value {
            CqlValue::Ascii(value) => match kind {
                SchemaFieldKind::String => Ok(Self::String(Some(value.to_owned()))),
                SchemaFieldKind::Bytes => Ok(Self::Bytes(Some(value.as_bytes().to_vec()))),
                _ => Err(Error::msg("Wrong value type")),
            },
            CqlValue::Boolean(value) => match kind {
                SchemaFieldKind::Bool => Ok(Self::Boolean(Some(*value))),
                SchemaFieldKind::Bytes => Ok(Self::Bytes(Some([(*value).into()].to_vec()))),
                _ => Err(Error::msg("Wrong value type")),
            },
            CqlValue::Blob(value) => match kind {
                SchemaFieldKind::Bytes => Ok(Self::Bytes(Some(value.to_owned()))),
                SchemaFieldKind::Json => match String::from_utf8(value.to_vec()) {
                    Ok(value) => Ok(Self::Json(Some(value))),
                    Err(_) => Err(Error::msg("Wrong value type")),
                },
                _ => Err(Error::msg("Wrong value type")),
            },
            CqlValue::Counter(value) => match kind {
                SchemaFieldKind::BigInt => Ok(Self::BigInteger(Some(value.0))),
                SchemaFieldKind::Bytes => Ok(Self::Bytes(Some(value.0.to_be_bytes().to_vec()))),
                _ => Err(Error::msg("Wrong value type")),
            },
            CqlValue::Decimal(value) => match value.to_f64() {
                Some(value) => match kind {
                    SchemaFieldKind::Double => Ok(Self::Double(Some(value))),
                    SchemaFieldKind::Bytes => Ok(Self::Bytes(Some(value.to_be_bytes().to_vec()))),
                    _ => Err(Error::msg("Wrong value type")),
                },
                None => Err(Error::msg("Wrong value type")),
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
                SchemaFieldKind::Bytes => Ok(Self::Bytes(Some(
                    (chrono::Duration::days((*value).into()) - chrono::Duration::days(1 << 31))
                        .num_milliseconds()
                        .to_be_bytes()
                        .to_vec(),
                ))),
                _ => Err(Error::msg("Wrong value type")),
            },
            CqlValue::Double(value) => match kind {
                SchemaFieldKind::Double => Ok(Self::Double(Some(*value))),
                SchemaFieldKind::Bytes => Ok(Self::Bytes(Some(value.to_be_bytes().to_vec()))),
                _ => Err(Error::msg("Wrong value type")),
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
                    let total_milli = value.nanoseconds / 1000000;
                    let milli = match u32::try_from(total_milli % 60) {
                        Ok(milli) => milli,
                        Err(_) => return Err(Error::msg("Wrong value type")),
                    };
                    let sec = match u32::try_from((total_milli / 1000) % 60) {
                        Ok(sec) => sec,
                        Err(_) => return Err(Error::msg("Wrong value type")),
                    };
                    let min: u32 = match u32::try_from((total_milli / (1000 * 60)) % 60) {
                        Ok(sec) => sec,
                        Err(_) => return Err(Error::msg("Wrong value type")),
                    };
                    let hour: u32 = match u32::try_from((total_milli / (1000 * 60 * 60)) % 60) {
                        Ok(sec) => sec,
                        Err(_) => return Err(Error::msg("Wrong value type")),
                    };
                    match NaiveTime::from_hms_milli_opt(hour, min, sec, milli) {
                        Some(data) => Ok(Self::Time(Some(data))),
                        None => Err(Error::msg("Wrong value type")),
                    }
                }
                SchemaFieldKind::DateTime => {
                    let nsecs = match u32::try_from(value.nanoseconds) {
                        Ok(nsecs) => nsecs,
                        Err(_) => return Err(Error::msg("Wrong value type")),
                    };
                    match DateTime::from_timestamp(0, nsecs) {
                        Some(value) => Ok(Self::DateTime(Some(value.into()))),
                        None => Err(Error::msg("Wrong value type")),
                    }
                }
                SchemaFieldKind::Timestamp => {
                    let nsecs = match u32::try_from(value.nanoseconds) {
                        Ok(nsecs) => nsecs,
                        Err(_) => return Err(Error::msg("Wrong value type")),
                    };
                    match DateTime::from_timestamp(0, nsecs) {
                        Some(value) => Ok(Self::Timestamp(Some(value.into()))),
                        None => Err(Error::msg("Wrong value type")),
                    }
                }
                SchemaFieldKind::Bytes => Ok(Self::Bytes(Some(
                    (chrono::Duration::nanoseconds(value.nanoseconds)
                        - chrono::Duration::days(1 << 31))
                    .num_milliseconds()
                    .to_be_bytes()
                    .to_vec(),
                ))),
                _ => Err(Error::msg("Wrong value type")),
            },
            CqlValue::Empty => Ok(Self::none(kind)),
            CqlValue::Float(value) => match kind {
                SchemaFieldKind::Float => Ok(Self::Float(Some(*value))),
                SchemaFieldKind::Double => Ok(Self::Double(Some((*value).into()))),
                SchemaFieldKind::Bytes => Ok(Self::Bytes(Some(value.to_be_bytes().to_vec()))),
                _ => Err(Error::msg("Wrong value type")),
            },
            CqlValue::Int(value) => match kind {
                SchemaFieldKind::Int => Ok(Self::Integer(Some(*value))),
                SchemaFieldKind::BigInt => Ok(Self::BigInteger(Some((*value).into()))),
                SchemaFieldKind::Bytes => Ok(Self::Bytes(Some(value.to_be_bytes().to_vec()))),
                _ => Err(Error::msg("Wrong value type")),
            },
            CqlValue::BigInt(value) => match kind {
                SchemaFieldKind::BigInt => Ok(Self::BigInteger(Some(*value))),
                SchemaFieldKind::Bytes => Ok(Self::Bytes(Some(value.to_be_bytes().to_vec()))),
                _ => Err(Error::msg("Wrong value type")),
            },
            CqlValue::Text(value) => match kind {
                SchemaFieldKind::String => Ok(Self::String(Some(value.to_owned()))),
                SchemaFieldKind::Bytes => Ok(Self::Bytes(Some(value.as_bytes().to_vec()))),
                _ => Err(Error::msg("Wrong value type")),
            },
            CqlValue::Timestamp(value) => match kind {
                SchemaFieldKind::DateTime => {
                    match DateTime::from_timestamp(value.num_seconds(), 0) {
                        Some(value) => Ok(Self::DateTime(Some(value.into()))),
                        None => Err(Error::msg("Wrong value type")),
                    }
                }
                SchemaFieldKind::Timestamp => {
                    match DateTime::from_timestamp(value.num_seconds(), 0) {
                        Some(value) => Ok(Self::Timestamp(Some(value.into()))),
                        None => Err(Error::msg("Wrong value type")),
                    }
                }
                SchemaFieldKind::Bytes => Ok(Self::Bytes(Some(
                    (chrono::Duration::milliseconds(value.num_milliseconds())
                        - chrono::Duration::days(1 << 31))
                    .num_milliseconds()
                    .to_be_bytes()
                    .to_vec(),
                ))),
                _ => Err(Error::msg("Wrong value type")),
            },
            CqlValue::Inet(value) => match kind {
                SchemaFieldKind::String => Ok(Self::String(Some(value.to_string()))),
                SchemaFieldKind::Bytes => {
                    Ok(Self::Bytes(Some(value.to_string().as_bytes().to_vec())))
                }
                _ => Err(Error::msg("Wrong value type")),
            },
            CqlValue::List(values) => match kind {
                SchemaFieldKind::Bytes => {
                    let mut data = Vec::new();
                    if values.serialize(&mut data).is_err() {
                        return Err(Error::msg("Wrong value type"));
                    }
                    Ok(Self::Bytes(Some(data)))
                }
                SchemaFieldKind::Json => {
                    let mut data = Vec::new();
                    if values.serialize(&mut data).is_err() {
                        return Err(Error::msg("Wrong value type"));
                    }
                    Ok(Self::Json(Some(serde_json::json!(data).to_string())))
                }
                _ => Err(Error::msg("Wrong value type")),
            },
            CqlValue::Map(values) => match kind {
                SchemaFieldKind::Bytes => {
                    let mut data = Vec::new();
                    if values.serialize(&mut data).is_err() {
                        return Err(Error::msg("Wrong value type"));
                    }
                    Ok(Self::Bytes(Some(data)))
                }
                SchemaFieldKind::Json => {
                    let mut data = Vec::new();
                    if values.serialize(&mut data).is_err() {
                        return Err(Error::msg("Wrong value type"));
                    }
                    Ok(Self::Json(Some(serde_json::json!(data).to_string())))
                }
                _ => Err(Error::msg("Wrong value type")),
            },
            CqlValue::Set(values) => match kind {
                SchemaFieldKind::Bytes => {
                    let mut data = Vec::new();
                    if values.serialize(&mut data).is_err() {
                        return Err(Error::msg("Wrong value type"));
                    }
                    Ok(Self::Bytes(Some(data)))
                }
                _ => Err(Error::msg("Wrong value type")),
            },
            CqlValue::UserDefinedType { fields, .. } => match kind {
                SchemaFieldKind::Bytes => {
                    let mut data = Vec::new();
                    if fields.serialize(&mut data).is_err() {
                        return Err(Error::msg("Wrong value type"));
                    }
                    Ok(Self::Bytes(Some(data)))
                }
                SchemaFieldKind::Json => {
                    let mut data = Vec::new();
                    if fields.serialize(&mut data).is_err() {
                        return Err(Error::msg("Wrong value type"));
                    }
                    Ok(Self::Json(Some(serde_json::json!(data).to_string())))
                }
                _ => Err(Error::msg("Wrong value type")),
            },
            CqlValue::SmallInt(value) => match kind {
                SchemaFieldKind::SmallInt => Ok(Self::SmallInteger(Some(*value))),
                SchemaFieldKind::Int => Ok(Self::Integer(Some((*value).into()))),
                SchemaFieldKind::BigInt => Ok(Self::BigInteger(Some((*value).into()))),
                SchemaFieldKind::Bytes => Ok(Self::Bytes(Some(value.to_be_bytes().to_vec()))),
                _ => Err(Error::msg("Wrong value type")),
            },
            CqlValue::TinyInt(value) => match kind {
                SchemaFieldKind::TinyInt => Ok(Self::TinyInteger(Some(*value))),
                SchemaFieldKind::SmallInt => Ok(Self::SmallInteger(Some((*value).into()))),
                SchemaFieldKind::Int => Ok(Self::Integer(Some((*value).into()))),
                SchemaFieldKind::BigInt => Ok(Self::BigInteger(Some((*value).into()))),
                SchemaFieldKind::Bytes => Ok(Self::Bytes(Some(value.to_be_bytes().to_vec()))),
                _ => Err(Error::msg("Wrong value type")),
            },
            CqlValue::Time(value) => match kind {
                SchemaFieldKind::Time => {
                    let total_milli = value.num_milliseconds();
                    let milli = match u32::try_from(total_milli % 60) {
                        Ok(milli) => milli,
                        Err(_) => return Err(Error::msg("Wrong value type")),
                    };
                    let sec = match u32::try_from((total_milli / 1000) % 60) {
                        Ok(sec) => sec,
                        Err(_) => return Err(Error::msg("Wrong value type")),
                    };
                    let min: u32 = match u32::try_from((total_milli / (1000 * 60)) % 60) {
                        Ok(sec) => sec,
                        Err(_) => return Err(Error::msg("Wrong value type")),
                    };
                    let hour: u32 = match u32::try_from((total_milli / (1000 * 60 * 60)) % 60) {
                        Ok(sec) => sec,
                        Err(_) => return Err(Error::msg("Wrong value type")),
                    };
                    match NaiveTime::from_hms_milli_opt(hour, min, sec, milli) {
                        Some(data) => Ok(Self::Time(Some(data))),
                        None => Err(Error::msg("Wrong value type")),
                    }
                }
                SchemaFieldKind::Bytes => Ok(Self::Bytes(Some(
                    (chrono::Duration::milliseconds(value.num_milliseconds())
                        - chrono::Duration::days(1 << 31))
                    .num_milliseconds()
                    .to_be_bytes()
                    .to_vec(),
                ))),
                _ => Err(Error::msg("Wrong value type")),
            },
            CqlValue::Timeuuid(value) => match kind {
                SchemaFieldKind::Bytes => Ok(Self::Bytes(Some(value.as_bytes().to_vec()))),
                SchemaFieldKind::Uuid => Ok(Self::Uuid(Some(*value))),
                _ => Err(Error::msg("Wrong value type")),
            },
            CqlValue::Tuple(value) => match kind {
                SchemaFieldKind::Bytes => {
                    let mut data = Vec::new();
                    if value.serialize(&mut data).is_err() {
                        return Err(Error::msg("Wrong value type"));
                    }
                    Ok(Self::Bytes(Some(data)))
                }
                _ => Err(Error::msg("Wrong value type")),
            },
            CqlValue::Uuid(value) => match kind {
                SchemaFieldKind::Uuid => Ok(Self::Uuid(Some(*value))),
                SchemaFieldKind::Bytes => Ok(Self::Bytes(Some(value.as_bytes().to_vec()))),
                _ => Err(Error::msg("Wrong value type")),
            },
            CqlValue::Varint(value) => match value.to_i64() {
                Some(value) => match kind {
                    SchemaFieldKind::BigInt => Ok(Self::BigInteger(Some(value))),
                    SchemaFieldKind::Bytes => Ok(Self::Bytes(Some(value.to_be_bytes().to_vec()))),
                    _ => Err(Error::msg("Wrong value type")),
                },
                None => Err(Error::msg("Wrong value type")),
            },
        }
    }

    pub fn to_scylladb_model(&self) -> Box<dyn Value> {
        match self {
            Self::Boolean(data) => Box::new(*data),
            Self::TinyInteger(data) => Box::new(*data),
            Self::SmallInteger(data) => Box::new(*data),
            Self::Integer(data) => Box::new(*data),
            Self::BigInteger(data) => Box::new(*data),
            Self::Float(data) => Box::new(*data),
            Self::Double(data) => Box::new(*data),
            Self::String(data) => Box::new(data.to_owned()),
            Self::Bytes(data) => Box::new(data.to_owned()),
            Self::Uuid(data) => Box::new(*data),
            Self::Date(data) => Box::new(*data),
            Self::Time(data) => Box::new(match data {
                Some(data) => Some(Time(*data - NaiveTime::MIN)),
                None => None,
            }),
            Self::DateTime(data) => Box::new(match data {
                Some(data) => Some(Timestamp(data.signed_duration_since(DateTime::UNIX_EPOCH))),
                None => None,
            }),
            Self::Timestamp(data) => Box::new(match data {
                Some(data) => Some(Timestamp(data.signed_duration_since(DateTime::UNIX_EPOCH))),
                None => None,
            }),
            Self::Json(data) => Box::new(data.to_owned()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RecordFilters(Vec<RecordFilter>);

impl RecordFilters {
    pub fn new(data: &Vec<RecordFilter>) -> Self {
        Self(data.to_vec())
    }

    pub fn scylladb_filter_query(
        &self,
        logical_operator: &Option<&str>,
        level: usize,
    ) -> Result<String> {
        if level > 1 {
            return Err(Error::msg(
                "ScyllaDB doesn't support filter query with level greater than 2",
            ));
        }
        let mut filter = String::new();
        for (idx, f) in self.0.iter().enumerate() {
            let op = f.op.to_uppercase();
            if let Some(child) = &f.child {
                if LOGICAL_OPERATOR.contains(&op.as_str()) {
                    if filter.len() > 0 {
                        filter += " ";
                    }
                    filter += &child.scylladb_filter_query(&Some(&op), level + 1)?;
                } else {
                    return Err(Error::msg(format!(
                        "Operator '{op}' is not supported as a logical operator in ScyllaDB"
                    )));
                }
            } else {
                let field = f.field.as_ref().unwrap();
                if COMPARISON_OPERATOR.contains(&op.as_str()) {
                    if filter.len() > 0 {
                        filter += " ";
                    }
                    filter += &format!("\"{}\" {} ?", field, &op);
                    if idx < self.0.len() - 1 {
                        if let Some(operator) = logical_operator {
                            if filter.len() > 0 {
                                filter += " ";
                            }
                            filter += operator
                        }
                    }
                } else {
                    return Err(Error::msg(format!(
                        "Operator '{op}' is not supported as a comparison operator in ScyllaDB"
                    )));
                }
            }
        }
        Ok(filter)
    }

    pub fn scylladb_values(&self) -> Vec<Box<dyn Value>> {
        let mut values = Vec::<Box<dyn Value>>::with_capacity(self.values_capacity());
        for f in &self.0 {
            if let Some(value) = &f.value {
                values.push(value.to_scylladb_model())
            }
            if let Some(child) = &f.child {
                values.append(&mut child.scylladb_values())
            }
        }
        values
    }

    fn values_capacity(&self) -> usize {
        let mut capacity = self.0.len();
        for f in &self.0 {
            if let Some(child) = &f.child {
                capacity += child.values_capacity()
            }
        }
        capacity
    }
}

#[derive(Clone, Debug)]
pub struct RecordFilter {
    field: Option<String>,
    op: String,
    value: Option<RecordColumnValue>,
    child: Option<RecordFilters>,
}

impl RecordFilter {
    pub fn new(
        field: &Option<String>,
        op: &str,
        value: &Option<RecordColumnValue>,
        child: &Option<RecordFilters>,
    ) -> Self {
        Self {
            field: field.to_owned(),
            op: op.to_owned(),
            value: value.clone(),
            child: child.clone(),
        }
    }

    pub fn field(&self) -> &Option<String> {
        &self.field
    }

    pub fn op(&self) -> &str {
        &self.op
    }

    pub fn value(&self) -> &Option<RecordColumnValue> {
        &self.value
    }

    pub fn child(&self) -> &Option<RecordFilters> {
        &self.child
    }
}

pub struct RecordOrder {
    field: String,
    kind: String,
}

impl RecordOrder {
    pub fn new(field: &str, kind: &str) -> Self {
        Self {
            field: field.to_owned(),
            kind: kind.to_owned(),
        }
    }

    pub fn field(&self) -> &str {
        &self.field
    }

    pub fn kind(&self) -> &str {
        &self.kind
    }
}

pub struct RecordPagination {
    limit: Option<i32>,
}

impl RecordPagination {
    pub fn new(limit: &Option<i32>) -> Self {
        Self { limit: *limit }
    }

    pub fn limit(&self) -> &Option<i32> {
        &self.limit
    }
}
