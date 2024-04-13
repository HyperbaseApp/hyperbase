use std::collections::hash_map::Keys;

use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use anyhow::{Error, Result};
use chrono::{Duration, Utc};
use hb_db_mysql::{
    db::MysqlDb,
    model::{
        collection::SchemaFieldPropsModel as SchemaFieldPropsMysqlModel,
        system::{
            COMPARISON_OPERATOR as MYSQL_COMPARISON_OPERATOR,
            LOGICAL_OPERATOR as MYSQL_LOGICAL_OPERATOR, ORDER_TYPE as MYSQL_ORDER_TYPE,
        },
    },
    query::{record as mysql_record, system::COUNT_TABLE as MYSQL_COUNT_TABLE},
};
use hb_db_postgresql::{
    db::PostgresDb,
    model::{
        collection::SchemaFieldPropsModel as SchemaFieldPropsPostgresModel,
        system::{
            COMPARISON_OPERATOR as POSTGRES_COMPARISON_OPERATOR,
            LOGICAL_OPERATOR as POSTGRES_LOGICAL_OPERATOR, ORDER_TYPE as POSTGRES_ORDER_TYPE,
        },
    },
    query::{record as postgres_record, system::COUNT_TABLE as POSTGRES_COUNT_TABLE},
};
use hb_db_scylladb::{
    db::ScyllaDb,
    model::{
        collection::SchemaFieldPropsModel as SchemaFieldPropsScyllaModel,
        system::{
            COMPARISON_OPERATOR as SCYLLA_COMPARISON_OPERATOR,
            LOGICAL_OPERATOR as SCYLLA_LOGICAL_OPERATOR, ORDER_TYPE as SCYLLA_ORDER_TYPE,
        },
    },
    query::{record as scylla_record, system::COUNT_TABLE as SCYLLA_COUNT_TABLE},
};
use hb_db_sqlite::{
    db::SqliteDb,
    model::{
        collection::SchemaFieldPropsModel as SchemaFieldPropsSqliteModel,
        system::{
            COMPARISON_OPERATOR as SQLITE_COMPARISON_OPERATOR,
            LOGICAL_OPERATOR as SQLITE_LOGICAL_OPERATOR, ORDER_TYPE as SQLITE_ORDER_TYPE,
        },
    },
    query::{record as sqlite_record, system::COUNT_TABLE as SQLITE_COUNT_TABLE},
};
use scylla::{
    frame::{response::result::CqlValue as ScyllaCqlValue, value::CqlTimestamp},
    serialize::value::SerializeCql,
};
use uuid::Uuid;

use crate::{
    collection::{CollectionDao, SchemaFieldProps},
    value::{ColumnKind, ColumnValue},
    Db,
};

pub struct RecordDao {
    table_name: String,
    data: HashMap<String, ColumnValue>,
}

impl RecordDao {
    pub fn new(created_by: &Uuid, collection_id: &Uuid, capacity: &Option<usize>) -> Self {
        let mut data = HashMap::with_capacity(match capacity {
            Some(capacity) => capacity + 3,
            None => 3,
        });
        data.insert("_id".to_owned(), ColumnValue::Uuid(Some(Uuid::now_v7())));
        data.insert(
            "_created_by".to_owned(),
            ColumnValue::Uuid(Some(*created_by)),
        );
        data.insert(
            "_updated_at".to_owned(),
            ColumnValue::Timestamp(Some(Utc::now())),
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

    pub fn data(&self) -> &HashMap<String, ColumnValue> {
        &self.data
    }

    pub fn get(&self, key: &str) -> Option<&ColumnValue> {
        self.data.get(key)
    }

    pub fn keys(&self) -> Keys<'_, String, ColumnValue> {
        self.data.keys()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn upsert(&mut self, key: &str, value: &ColumnValue) {
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
            Db::PostgresqlDb(db) => {
                Self::postgresdb_create_table(
                    db,
                    collection.id(),
                    &collection
                        .schema_fields()
                        .iter()
                        .map(|(field_name, field_props)| {
                            (field_name.clone(), field_props.to_postgresdb_model())
                        })
                        .collect::<HashMap<_, _>>(),
                )
                .await
            }
            Db::MysqlDb(db) => {
                Self::mysqldb_create_table(
                    db,
                    collection.id(),
                    &collection
                        .schema_fields()
                        .iter()
                        .map(|(field_name, field_props)| {
                            (field_name.clone(), field_props.to_mysqldb_model())
                        })
                        .collect::<HashMap<_, _>>(),
                )
                .await
            }
            Db::SqliteDb(db) => {
                Self::sqlitedb_create_table(
                    db,
                    collection.id(),
                    &collection
                        .schema_fields()
                        .iter()
                        .map(|(field_name, field_props)| {
                            (field_name.clone(), field_props.to_sqlitedb_model())
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
            Db::PostgresqlDb(db) => Self::postgresdb_drop_table(db, collection_id).await,
            Db::MysqlDb(db) => Self::mysqldb_drop_table(db, collection_id).await,
            Db::SqliteDb(db) => Self::sqlite_drop_table(db, collection_id).await,
        }
    }

    pub async fn db_check_table_existence(db: &Db, collection_id: &Uuid) -> Result<bool> {
        match db {
            Db::ScyllaDb(db) => Self::scylladb_check_table_existence(db, collection_id).await,
            Db::PostgresqlDb(db) => Self::postgresdb_check_table_existence(db, collection_id).await,
            Db::MysqlDb(db) => Self::mysqldb_check_table_existence(db, collection_id).await,
            Db::SqliteDb(db) => Self::sqlitedb_check_table_existence(db, collection_id).await,
        }
    }

    pub async fn db_check_table_must_exist(db: &Db, collection_id: &Uuid) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => {
                match Self::scylladb_check_table_existence(db, collection_id).await? {
                    true => Ok(()),
                    false => Err(Error::msg(format!(
                        "Collection '{collection_id}' doesn't exist"
                    ))),
                }
            }
            Db::PostgresqlDb(db) => {
                match Self::postgresdb_check_table_existence(db, collection_id).await? {
                    true => Ok(()),
                    false => Err(Error::msg(format!(
                        "Collection '{collection_id}' doesn't exist"
                    ))),
                }
            }
            Db::MysqlDb(db) => {
                match Self::mysqldb_check_table_existence(db, collection_id).await? {
                    true => Ok(()),
                    false => Err(Error::msg(format!(
                        "Collection '{collection_id}' doesn't exist"
                    ))),
                }
            }
            Db::SqliteDb(db) => {
                match Self::sqlitedb_check_table_existence(db, collection_id).await? {
                    true => Ok(()),
                    false => Err(Error::msg(format!(
                        "Collection '{collection_id}' doesn't exist"
                    ))),
                }
            }
        }
    }

    pub async fn db_add_columns(
        db: &Db,
        collection_id: &Uuid,
        columns: &HashMap<String, SchemaFieldProps>,
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
            Db::PostgresqlDb(db) => {
                Self::postgresdb_add_columns(
                    db,
                    collection_id,
                    &columns
                        .iter()
                        .map(|(col, col_props)| (col.to_owned(), col_props.to_postgresdb_model()))
                        .collect(),
                )
                .await
            }
            Db::MysqlDb(db) => {
                Self::mysqldb_add_columns(
                    db,
                    collection_id,
                    &columns
                        .iter()
                        .map(|(col, col_props)| (col.to_owned(), col_props.to_mysqldb_model()))
                        .collect(),
                )
                .await
            }
            Db::SqliteDb(db) => {
                Self::sqlitedb_add_columns(
                    db,
                    collection_id,
                    &columns
                        .iter()
                        .map(|(col, col_props)| (col.to_owned(), col_props.to_sqlitedb_model()))
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
            Db::PostgresqlDb(db) => {
                Self::postgresdb_drop_columns(db, collection_id, column_names).await
            }
            Db::MysqlDb(db) => Self::mysqldb_drop_columns(db, collection_id, column_names).await,
            Db::SqliteDb(db) => Self::sqlitedb_drop_columns(db, collection_id, column_names).await,
        }
    }

    pub async fn db_change_columns_type(
        db: &Db,
        collection_id: &Uuid,
        columns: &HashMap<String, SchemaFieldProps>,
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
            Db::PostgresqlDb(db) => {
                Self::postgresdb_change_columns_type(
                    db,
                    collection_id,
                    &columns
                        .iter()
                        .map(|(col, col_props)| (col.to_owned(), col_props.to_postgresdb_model()))
                        .collect(),
                )
                .await
            }
            Db::MysqlDb(db) => {
                Self::mysqldb_change_columns_type(
                    db,
                    collection_id,
                    &columns
                        .iter()
                        .map(|(col, col_props)| (col.to_owned(), col_props.to_mysqldb_model()))
                        .collect(),
                )
                .await
            }
            Db::SqliteDb(db) => {
                Self::sqlitedb_change_columns_type(
                    db,
                    collection_id,
                    &columns
                        .iter()
                        .map(|(col, col_props)| (col.to_owned(), col_props.to_sqlitedb_model()))
                        .collect(),
                )
                .await
            }
        }
    }

    pub async fn db_create_index(db: &Db, collection_id: &Uuid, index: &str) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => Self::scylladb_create_index(db, collection_id, index).await,
            Db::PostgresqlDb(db) => Self::postgresdb_create_index(db, collection_id, index).await,
            Db::MysqlDb(db) => Self::mysqldb_create_index(db, collection_id, index).await,
            Db::SqliteDb(db) => Self::sqlitedb_create_index(db, collection_id, index).await,
        }
    }

    pub async fn db_create_unique_index(db: &Db, collection_id: &Uuid, index: &str) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => Self::scylladb_create_index(db, collection_id, index).await,
            Db::PostgresqlDb(db) => {
                Self::postgresdb_create_unique_index(db, collection_id, index).await
            }
            Db::MysqlDb(db) => Self::mysqldb_create_unique_index(db, collection_id, index).await,
            Db::SqliteDb(db) => Self::sqlitedb_create_unique_index(db, collection_id, index).await,
        }
    }

    pub async fn db_drop_index(db: &Db, collection_id: &Uuid, index: &str) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => Self::scylladb_drop_index(db, collection_id, index).await,
            Db::PostgresqlDb(db) => Self::postgresdb_drop_index(db, collection_id, index).await,
            Db::MysqlDb(db) => Self::mysqldb_drop_index(db, collection_id, index).await,
            Db::SqliteDb(db) => Self::sqlitedb_drop_index(db, collection_id, index).await,
        }
    }

    pub async fn db_drop_unique_index(db: &Db, collection_id: &Uuid, index: &str) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => Self::scylladb_drop_index(db, collection_id, index).await,
            Db::PostgresqlDb(db) => {
                Self::postgresdb_drop_unique_index(db, collection_id, index).await
            }
            Db::MysqlDb(db) => Self::mysqldb_drop_unique_index(db, collection_id, index).await,
            Db::SqliteDb(db) => Self::sqlitedb_drop_unique_index(db, collection_id, index).await,
        }
    }

    pub async fn db_insert(&self, db: &Db) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => Self::scylladb_insert(self, db).await,
            Db::PostgresqlDb(db) => Self::postgresdb_insert(self, db).await,
            Db::MysqlDb(db) => Self::mysqldb_insert(self, db).await,
            Db::SqliteDb(db) => Self::sqlitedb_insert(self, db).await,
        }
    }

    pub async fn db_select(
        db: &Db,
        id: &Uuid,
        created_by: &Option<Uuid>,
        fields: &HashSet<&str>,
        collection_data: &CollectionDao,
        is_admin: &bool,
    ) -> Result<Self> {
        if let Some(ttl_seconds) = collection_data.opt_ttl() {
            Self::db_delete_expired(db, collection_data.id(), ttl_seconds).await?;
        }
        match db {
            Db::ScyllaDb(db) => {
                let table_name = Self::new_table_name(collection_data.id());

                let mut columns;
                if fields.len() > 0 {
                    columns = Vec::with_capacity(fields.len());
                    for column in fields {
                        for column in ["_id", "_created_by", "_updated_at"] {
                            columns.push(column);
                        }
                        if collection_data.schema_fields().get(*column).is_some() {
                            columns.push(*column);
                        }
                    }
                } else {
                    columns = Vec::with_capacity(collection_data.schema_fields().len() + 3);
                    for column in ["_id", "_created_by", "_updated_at"] {
                        columns.push(column);
                    }
                    for column in collection_data.schema_fields().keys() {
                        columns.push(column);
                    }
                }

                let scylladb_data =
                    Self::scylladb_select(db, &table_name, &columns, id, created_by).await?;

                let mut data = HashMap::with_capacity(scylladb_data.len());
                for (idx, value) in scylladb_data.iter().enumerate() {
                    let kind;
                    if ["_id", "_created_by"].contains(&columns[idx]) {
                        kind = &ColumnKind::Uuid;
                    } else if columns[idx] == "_updated_at" {
                        kind = &ColumnKind::Timestamp;
                    } else {
                        let schema_field = collection_data
                            .schema_fields()
                            .get(columns[idx])
                            .ok_or_else(|| {
                                Error::msg(format!(
                                    "Field {} is not found in the collection",
                                    columns[idx]
                                ))
                            })?;
                        if !*is_admin && *schema_field.hidden() {
                            continue;
                        }
                        kind = schema_field.kind();
                    }
                    match value {
                        Some(value) => match ColumnValue::from_scylladb_model(kind, value) {
                            Ok(value) => data.insert(columns[idx].to_owned(), value),
                            Err(err) => return Err(err),
                        },
                        None => data.insert(columns[idx].to_owned(), ColumnValue::none(kind)),
                    };
                }

                Ok(Self { table_name, data })
            }
            Db::PostgresqlDb(db) => {
                let table_name = Self::new_table_name(collection_data.id());

                let mut columns;
                if fields.len() > 0 {
                    columns = Vec::with_capacity(fields.len());
                    for column in fields {
                        for column in ["_id", "_created_by", "_updated_at"] {
                            columns.push(column);
                        }
                        if collection_data.schema_fields().get(*column).is_some() {
                            columns.push(*column);
                        }
                    }
                } else {
                    columns = Vec::with_capacity(collection_data.schema_fields().len() + 3);
                    columns.append(&mut Vec::from(["_id", "_created_by", "_updated_at"]));
                    for column in collection_data.schema_fields().keys() {
                        columns.push(column);
                    }
                }

                let postgresdb_data =
                    Self::postgresdb_select(db, &table_name, &columns, id, created_by).await?;

                let mut data = HashMap::with_capacity(columns.len());
                for column in columns {
                    let kind;
                    if ["_id", "_created_by"].contains(&column) {
                        kind = &ColumnKind::Uuid;
                    } else if column == "_updated_at" {
                        kind = &ColumnKind::Timestamp;
                    } else {
                        let schema_field =
                            collection_data.schema_fields().get(column).ok_or_else(|| {
                                Error::msg(format!("Field {column} is not found in the collection"))
                            })?;
                        if !*is_admin && *schema_field.hidden() {
                            continue;
                        }
                        kind = schema_field.kind();
                    }
                    data.insert(
                        column.to_owned(),
                        ColumnValue::from_postgresdb_model(kind, column, &postgresdb_data)?,
                    );
                }

                Ok(Self { table_name, data })
            }
            Db::MysqlDb(db) => {
                let table_name = Self::new_table_name(collection_data.id());

                let mut columns;
                if fields.len() > 0 {
                    columns = Vec::with_capacity(fields.len());
                    for column in fields {
                        for column in ["_id", "_created_by", "_updated_at"] {
                            columns.push(column);
                        }
                        if collection_data.schema_fields().get(*column).is_some() {
                            columns.push(*column);
                        }
                    }
                } else {
                    columns = Vec::with_capacity(collection_data.schema_fields().len() + 3);
                    columns.append(&mut Vec::from(["_id", "_created_by", "_updated_at"]));
                    for column in collection_data.schema_fields().keys() {
                        columns.push(column);
                    }
                }

                let mysqldb_data =
                    Self::mysqldb_select(db, &table_name, &columns, id, created_by).await?;

                let mut data = HashMap::with_capacity(columns.len());
                for column in columns {
                    let kind;
                    if ["_id", "_created_by"].contains(&column) {
                        kind = &ColumnKind::Uuid;
                    } else if column == "_updated_at" {
                        kind = &ColumnKind::Timestamp;
                    } else {
                        let schema_field =
                            collection_data.schema_fields().get(column).ok_or_else(|| {
                                Error::msg(format!("Field {column} is not found in the collection"))
                            })?;
                        if !*is_admin && *schema_field.hidden() {
                            continue;
                        }
                        kind = schema_field.kind();
                    }
                    data.insert(
                        column.to_owned(),
                        ColumnValue::from_mysqldb_model(kind, column, &mysqldb_data)?,
                    );
                }

                Ok(Self { table_name, data })
            }
            Db::SqliteDb(db) => {
                let table_name = Self::new_table_name(collection_data.id());

                let mut columns;
                if fields.len() > 0 {
                    columns = Vec::with_capacity(fields.len());
                    for column in fields {
                        for column in ["_id", "_created_by", "_updated_at"] {
                            columns.push(column);
                        }
                        if collection_data.schema_fields().get(*column).is_some() {
                            columns.push(*column);
                        }
                    }
                } else {
                    columns = Vec::with_capacity(collection_data.schema_fields().len() + 3);
                    columns.append(&mut Vec::from(["_id", "_created_by", "_updated_at"]));
                    for column in collection_data.schema_fields().keys() {
                        columns.push(column);
                    }
                }

                let sqlitedb_data =
                    Self::sqlitedb_select(db, &table_name, &columns, id, created_by).await?;

                let mut data = HashMap::with_capacity(columns.len());
                for column in columns {
                    let kind;
                    if ["_id", "_created_by"].contains(&column) {
                        kind = &ColumnKind::Uuid;
                    } else if column == "_updated_at" {
                        kind = &ColumnKind::Timestamp;
                    } else {
                        let schema_field =
                            collection_data.schema_fields().get(column).ok_or_else(|| {
                                Error::msg(format!("Field {column} is not found in the collection"))
                            })?;
                        if !*is_admin && *schema_field.hidden() {
                            continue;
                        }
                        kind = schema_field.kind();
                    }
                    data.insert(
                        column.to_owned(),
                        ColumnValue::from_sqlitedb_model(kind, column, &sqlitedb_data)?,
                    );
                }

                Ok(Self { table_name, data })
            }
        }
    }

    pub async fn db_select_many(
        db: &Db,
        fields: &HashSet<&str>,
        collection_data: &CollectionDao,
        created_by: &Option<Uuid>,
        filters: &RecordFilters,
        groups: &Vec<&str>,
        orders: &Vec<RecordOrder>,
        pagination: &RecordPagination,
        is_admin: &bool,
    ) -> Result<(Vec<Self>, i64)> {
        if let Some(ttl_seconds) = collection_data.opt_ttl() {
            Self::db_delete_expired(db, collection_data.id(), ttl_seconds).await?;
        }
        match db {
            Db::ScyllaDb(db) => {
                let table_name = Self::new_table_name(collection_data.id());

                let mut columns;
                if fields.len() > 0 {
                    columns = Vec::with_capacity(fields.len());
                    for column in fields {
                        if collection_data.schema_fields().get(*column).is_some() {
                            columns.push(*column);
                        }
                    }
                } else {
                    columns = Vec::with_capacity(collection_data.schema_fields().len() + 3);
                    for column in ["_id", "_created_by", "_updated_at"] {
                        columns.push(column);
                    }
                    for column in collection_data.schema_fields().keys() {
                        columns.push(column);
                    }
                }

                let (scylladb_data_many, total) = Self::scylladb_select_many(
                    db,
                    &table_name,
                    &columns,
                    created_by,
                    filters,
                    groups,
                    orders,
                    pagination,
                )
                .await?;

                let mut data_many = Vec::with_capacity(scylladb_data_many.len());
                for scylladb_data in scylladb_data_many {
                    let mut data = HashMap::with_capacity(scylladb_data.len());
                    for (idx, value) in scylladb_data.iter().enumerate() {
                        let kind;
                        if ["_id", "_created_by"].contains(&columns[idx]) {
                            kind = &ColumnKind::Uuid;
                        } else if columns[idx] == "_updated_at" {
                            kind = &ColumnKind::Timestamp;
                        } else {
                            let schema_field = collection_data
                                .schema_fields()
                                .get(columns[idx])
                                .ok_or_else(|| {
                                    Error::msg(format!(
                                        "Field {} is not found in the collection",
                                        columns[idx]
                                    ))
                                })?;
                            if !*is_admin && *schema_field.hidden() {
                                continue;
                            }
                            kind = schema_field.kind();
                        }
                        match value {
                            Some(value) => match ColumnValue::from_scylladb_model(kind, value) {
                                Ok(value) => data.insert(columns[idx].to_owned(), value),
                                Err(err) => return Err(err),
                            },
                            None => data.insert(columns[idx].to_owned(), ColumnValue::none(kind)),
                        };
                    }
                    if let Some(created_by) = created_by {
                        if let Some(created_by_data) = data.get("_created_by") {
                            if let ColumnValue::Uuid(id) = created_by_data {
                                if let Some(id) = id {
                                    if *created_by != *id {
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                    data_many.push(Self {
                        table_name: table_name.to_owned(),
                        data,
                    });
                }

                Ok((data_many, total))
            }
            Db::PostgresqlDb(db) => {
                let table_name = Self::new_table_name(collection_data.id());

                let mut columns;
                if fields.len() > 0 {
                    columns = Vec::with_capacity(fields.len());
                    for column in fields {
                        columns.push(*column);
                    }
                } else {
                    columns = Vec::with_capacity(collection_data.schema_fields().len() + 3);
                    columns.append(&mut Vec::from(["_id", "_created_by", "_updated_at"]));
                    for column in collection_data.schema_fields().keys() {
                        columns.push(column);
                    }
                }

                let (postgres_data_many, total) = Self::postgresdb_select_many(
                    db,
                    &table_name,
                    &columns,
                    created_by,
                    filters,
                    groups,
                    orders,
                    pagination,
                )
                .await?;

                let mut data_many = Vec::with_capacity(postgres_data_many.len());
                for postgres_data in &postgres_data_many {
                    let mut data = HashMap::with_capacity(columns.len());
                    for column in &columns {
                        let kind;
                        if ["_id", "_created_by"].contains(&column) {
                            kind = &ColumnKind::Uuid;
                        } else if *column == "_updated_at" {
                            kind = &ColumnKind::Timestamp;
                        } else {
                            let schema_field = collection_data
                                .schema_fields()
                                .get(*column)
                                .ok_or_else(|| {
                                    Error::msg(format!(
                                        "Field {column} is not found in the collection"
                                    ))
                                })?;
                            if !*is_admin && *schema_field.hidden() {
                                continue;
                            }
                            kind = schema_field.kind();
                        }
                        data.insert(
                            column.to_string(),
                            ColumnValue::from_postgresdb_model(kind, column, &postgres_data)?,
                        );
                    }
                    data_many.push(Self {
                        table_name: table_name.to_owned(),
                        data,
                    })
                }

                Ok((data_many, total))
            }
            Db::MysqlDb(db) => {
                let table_name = Self::new_table_name(collection_data.id());

                let mut columns;
                if fields.len() > 0 {
                    columns = Vec::with_capacity(fields.len());
                    for column in fields {
                        columns.push(*column);
                    }
                } else {
                    columns = Vec::with_capacity(collection_data.schema_fields().len() + 3);
                    columns.append(&mut Vec::from(["_id", "_created_by", "_updated_at"]));
                    for column in collection_data.schema_fields().keys() {
                        columns.push(column);
                    }
                }

                let (mysql_data_many, total) = Self::mysqldb_select_many(
                    db,
                    &table_name,
                    &columns,
                    created_by,
                    filters,
                    groups,
                    orders,
                    pagination,
                )
                .await?;

                let mut data_many = Vec::with_capacity(mysql_data_many.len());
                for mysql_data in &mysql_data_many {
                    let mut data = HashMap::with_capacity(columns.len());
                    for column in &columns {
                        let kind;
                        if ["_id", "_created_by"].contains(&column) {
                            kind = &ColumnKind::Uuid;
                        } else if *column == "_updated_at" {
                            kind = &ColumnKind::Timestamp;
                        } else {
                            let schema_field = collection_data
                                .schema_fields()
                                .get(*column)
                                .ok_or_else(|| {
                                    Error::msg(format!(
                                        "Field {column} is not found in the collection"
                                    ))
                                })?;
                            if !*is_admin && *schema_field.hidden() {
                                continue;
                            }
                            kind = schema_field.kind();
                        }
                        data.insert(
                            column.to_string(),
                            ColumnValue::from_mysqldb_model(kind, column, &mysql_data)?,
                        );
                    }
                    data_many.push(Self {
                        table_name: table_name.to_owned(),
                        data,
                    })
                }

                Ok((data_many, total))
            }
            Db::SqliteDb(db) => {
                let table_name = Self::new_table_name(collection_data.id());

                let mut columns;
                if fields.len() > 0 {
                    columns = Vec::with_capacity(fields.len());
                    for column in fields {
                        columns.push(*column);
                    }
                } else {
                    columns = Vec::with_capacity(collection_data.schema_fields().len() + 3);
                    columns.append(&mut Vec::from(["_id", "_created_by", "_updated_at"]));
                    for column in collection_data.schema_fields().keys() {
                        columns.push(column);
                    }
                }

                let (sqlite_data_many, total) = Self::sqlitedb_select_many(
                    db,
                    &table_name,
                    &columns,
                    created_by,
                    filters,
                    groups,
                    orders,
                    pagination,
                )
                .await?;

                let mut data_many = Vec::with_capacity(sqlite_data_many.len());
                for sqlite_data in &sqlite_data_many {
                    let mut data = HashMap::with_capacity(columns.len());
                    for column in &columns {
                        let kind;
                        if ["_id", "_created_by"].contains(&column) {
                            kind = &ColumnKind::Uuid;
                        } else if *column == "_updated_at" {
                            kind = &ColumnKind::Timestamp;
                        } else {
                            let schema_field = collection_data
                                .schema_fields()
                                .get(*column)
                                .ok_or_else(|| {
                                    Error::msg(format!(
                                        "Field {column} is not found in the collection"
                                    ))
                                })?;
                            if !*is_admin && *schema_field.hidden() {
                                continue;
                            }
                            kind = schema_field.kind();
                        }
                        data.insert(
                            column.to_string(),
                            ColumnValue::from_sqlitedb_model(kind, column, &sqlite_data)?,
                        );
                    }
                    data_many.push(Self {
                        table_name: table_name.to_owned(),
                        data,
                    })
                }

                Ok((data_many, total))
            }
        }
    }

    pub async fn db_update(&mut self, db: &Db) -> Result<()> {
        self.data.insert(
            "_updated_at".to_owned(),
            ColumnValue::Timestamp(Some(Utc::now())),
        );
        match db {
            Db::ScyllaDb(db) => Self::scylladb_update(self, db).await,
            Db::PostgresqlDb(db) => Self::postgresdb_update(self, db).await,
            Db::MysqlDb(db) => Self::mysqldb_update(self, db).await,
            Db::SqliteDb(db) => Self::sqlitedb_update(self, db).await,
        }
    }

    pub async fn db_delete(
        db: &Db,
        collection_id: &Uuid,
        id: &Uuid,
        created_by: &Option<Uuid>,
    ) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => Self::scylladb_delete(db, collection_id, id, created_by).await,
            Db::PostgresqlDb(db) => {
                Self::postgresdb_delete(db, collection_id, id, created_by).await
            }
            Db::MysqlDb(db) => Self::mysqldb_delete(db, collection_id, id, created_by).await,
            Db::SqliteDb(db) => Self::sqlitedb_delete(db, collection_id, id, created_by).await,
        }
    }

    async fn db_delete_expired(db: &Db, collection_id: &Uuid, ttl_seconds: &i64) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => Self::scylladb_delete_expired(db, collection_id, ttl_seconds).await,
            Db::PostgresqlDb(db) => {
                Self::postgresdb_delete_expired(db, collection_id, ttl_seconds).await
            }
            Db::MysqlDb(db) => Self::mysqldb_delete_expired(db, collection_id, ttl_seconds).await,
            Db::SqliteDb(db) => Self::sqlitedb_delete_expired(db, collection_id, ttl_seconds).await,
        }
    }

    async fn scylladb_create_table(
        db: &ScyllaDb,
        collection_id: &Uuid,
        schema_fields: &HashMap<String, SchemaFieldPropsScyllaModel>,
    ) -> Result<()> {
        db.session_query(
            &scylla_record::create_table(&Self::new_table_name(collection_id), schema_fields),
            &[],
        )
        .await?;
        Ok(())
    }

    async fn scylladb_drop_table(db: &ScyllaDb, collection_id: &Uuid) -> Result<()> {
        db.session_query(
            &scylla_record::drop_table(&RecordDao::new_table_name(collection_id)),
            &[],
        )
        .await?;
        Ok(())
    }

    async fn scylladb_check_table_existence(db: &ScyllaDb, collection_id: &Uuid) -> Result<bool> {
        Ok(db
            .session_query(
                SCYLLA_COUNT_TABLE,
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
            &scylla_record::add_columns(&Self::new_table_name(collection_id), columns),
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
            &scylla_record::drop_columns(&Self::new_table_name(collection_id), column_names),
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
            &scylla_record::change_columns_type(&Self::new_table_name(collection_id), columns),
            &[],
        )
        .await?;
        Ok(())
    }

    async fn scylladb_create_index(db: &ScyllaDb, collection_id: &Uuid, index: &str) -> Result<()> {
        db.session_query(
            &scylla_record::create_index(&Self::new_table_name(collection_id), index),
            &[],
        )
        .await?;
        Ok(())
    }

    async fn scylladb_drop_index(db: &ScyllaDb, collection_id: &Uuid, index: &str) -> Result<()> {
        db.session_query(
            &scylla_record::drop_index(&Self::new_table_name(collection_id), index),
            &[],
        )
        .await?;
        Ok(())
    }

    async fn scylladb_insert(&self, db: &ScyllaDb) -> Result<()> {
        let mut columns = Vec::with_capacity(self.data.len());
        let mut values = Vec::with_capacity(self.data.len());
        for (col, val) in &self.data {
            columns.push(col.as_str());
            values.push(val.to_scylladb_model()?);
        }
        db.execute(&scylla_record::insert(&self.table_name, &columns), &values)
            .await?;
        Ok(())
    }

    async fn scylladb_select(
        db: &ScyllaDb,
        table_name: &str,
        columns: &Vec<&str>,
        id: &Uuid,
        created_by: &Option<Uuid>,
    ) -> Result<Vec<Option<ScyllaCqlValue>>> {
        Ok(if let Some(created_by) = created_by {
            db.execute(
                &scylla_record::select_by_id_and_created_by(table_name, columns),
                [id, created_by].as_ref(),
            )
            .await?
            .first_row()?
            .columns
        } else {
            db.execute(&scylla_record::select(table_name, columns), [id].as_ref())
                .await?
                .first_row()?
                .columns
        })
    }

    async fn scylladb_select_many(
        db: &ScyllaDb,
        table_name: &str,
        columns: &Vec<&str>,
        created_by: &Option<Uuid>,
        filters: &RecordFilters,
        groups: &Vec<&str>,
        orders: &Vec<RecordOrder>,
        pagination: &RecordPagination,
    ) -> Result<(Vec<Vec<Option<ScyllaCqlValue>>>, i64)> {
        let mut filter = filters.scylladb_filter_query(&None, 0)?;
        if created_by.is_some() {
            if filter.len() > 0 {
                filter = format!("\"_created_by\" = ? AND ({filter})");
            } else {
                filter = format!("\"_created_by\" = ?");
            }
        }

        let mut order = Vec::with_capacity(orders.len());
        for o in orders {
            if SCYLLA_ORDER_TYPE.contains(&o.kind.to_uppercase().as_str()) {
                order.push((o.field.as_str(), o.kind.as_str()));
            } else {
                return Err(Error::msg(format!(
                    "Order type '{}' is not supported",
                    &o.kind
                )));
            }
        }

        let mut values = filters.scylladb_values()?;
        let mut total_values = filters.scylladb_values()?;
        if let Some(created_by) = created_by {
            values.insert(0, Box::new(created_by));
            total_values.insert(0, Box::new(created_by));
        }
        if let Some(limit) = pagination.limit() {
            values.push(Box::new(limit));
        }

        let query_select_many = scylla_record::select_many(
            table_name,
            columns,
            &filter,
            groups,
            &order,
            &pagination.limit().is_some(),
        );
        let query_total = scylla_record::count(table_name, &filter);

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
            if col != "_id" && col != "_updated_at" {
                columns.push(col.as_str());
                values.push(val.to_scylladb_model()?);
            }
        }
        match self.data.get("_updated_at") {
            Some(updated_at) => {
                columns.push("_updated_at");
                values.push(updated_at.to_scylladb_model()?);
            }
            None => return Err(Error::msg("_updated_at is undefined")),
        }
        match self.data.get("_id") {
            Some(id) => values.push(id.to_scylladb_model()?),
            None => return Err(Error::msg("_id is undefined")),
        }
        db.execute(&scylla_record::update(&self.table_name, &columns), &values)
            .await?;
        Ok(())
    }

    async fn scylladb_delete(
        db: &ScyllaDb,
        collection_id: &Uuid,
        id: &Uuid,
        created_by: &Option<Uuid>,
    ) -> Result<()> {
        let mut column = HashSet::<String>::with_capacity(2);
        column.insert("_id".to_owned());
        if let Some(created_by) = created_by {
            column.insert("_created_by".to_owned());
            db.execute(
                &scylla_record::delete(&Self::new_table_name(collection_id), &column),
                [id, created_by].as_ref(),
            )
            .await?;
        } else {
            db.execute(
                &scylla_record::delete(&Self::new_table_name(collection_id), &column),
                [id].as_ref(),
            )
            .await?;
        }
        Ok(())
    }

    async fn scylladb_delete_expired(
        db: &ScyllaDb,
        collection_id: &Uuid,
        ttl_seconds: &i64,
    ) -> Result<()> {
        db.execute(
            &scylla_record::delete_expired(&Self::new_table_name(collection_id)),
            [CqlTimestamp(
                Utc::now()
                    .checked_sub_signed(
                        Duration::try_seconds(*ttl_seconds)
                            .ok_or_else(|| Error::msg("collection ttl is out of range."))?,
                    )
                    .ok_or_else(|| Error::msg("collection ttl is out of range."))?
                    .timestamp_millis(),
            )]
            .as_ref(),
        )
        .await?;
        Ok(())
    }

    async fn postgresdb_create_table(
        db: &PostgresDb,
        collection_id: &Uuid,
        schema_fields: &HashMap<String, SchemaFieldPropsPostgresModel>,
    ) -> Result<()> {
        db.execute_unprepared(sqlx::query(&postgres_record::create_table(
            &Self::new_table_name(collection_id),
            schema_fields,
        )))
        .await?;
        Ok(())
    }

    async fn postgresdb_drop_table(db: &PostgresDb, collection_id: &Uuid) -> Result<()> {
        db.execute_unprepared(sqlx::query(&postgres_record::drop_table(
            &Self::new_table_name(collection_id),
        )))
        .await?;
        Ok(())
    }

    async fn postgresdb_check_table_existence(
        db: &PostgresDb,
        collection_id: &Uuid,
    ) -> Result<bool> {
        Ok(db
            .fetch_one_unprepared::<(i64,)>(
                sqlx::query_as(POSTGRES_COUNT_TABLE)
                    .bind(&RecordDao::new_table_name(collection_id)),
            )
            .await?
            .0
            > 0)
    }

    async fn postgresdb_add_columns(
        db: &PostgresDb,
        collection_id: &Uuid,
        columns: &HashMap<String, SchemaFieldPropsPostgresModel>,
    ) -> Result<()> {
        db.execute_unprepared(sqlx::query(&postgres_record::add_columns(
            &Self::new_table_name(collection_id),
            columns,
        )))
        .await?;
        Ok(())
    }

    async fn postgresdb_drop_columns(
        db: &PostgresDb,
        collection_id: &Uuid,
        column_names: &HashSet<String>,
    ) -> Result<()> {
        db.execute_unprepared(sqlx::query(&postgres_record::drop_columns(
            &Self::new_table_name(collection_id),
            column_names,
        )))
        .await?;
        Ok(())
    }

    async fn postgresdb_change_columns_type(
        db: &PostgresDb,
        collection_id: &Uuid,
        columns: &HashMap<String, SchemaFieldPropsPostgresModel>,
    ) -> Result<()> {
        db.execute_unprepared(sqlx::query(&postgres_record::change_columns_type(
            &Self::new_table_name(collection_id),
            columns,
        )))
        .await?;
        Ok(())
    }

    async fn postgresdb_create_index(
        db: &PostgresDb,
        collection_id: &Uuid,
        index: &str,
    ) -> Result<()> {
        db.execute_unprepared(sqlx::query(&postgres_record::create_index(
            &Self::new_table_name(collection_id),
            index,
        )))
        .await?;
        Ok(())
    }

    async fn postgresdb_create_unique_index(
        db: &PostgresDb,
        collection_id: &Uuid,
        index: &str,
    ) -> Result<()> {
        db.execute_unprepared(sqlx::query(&postgres_record::create_unique_index(
            &Self::new_table_name(collection_id),
            index,
        )))
        .await?;
        Ok(())
    }

    async fn postgresdb_drop_index(
        db: &PostgresDb,
        collection_id: &Uuid,
        index: &str,
    ) -> Result<()> {
        db.execute_unprepared(sqlx::query(&postgres_record::drop_index(
            &Self::new_table_name(collection_id),
            index,
        )))
        .await?;
        Ok(())
    }

    async fn postgresdb_drop_unique_index(
        db: &PostgresDb,
        collection_id: &Uuid,
        index: &str,
    ) -> Result<()> {
        db.execute_unprepared(sqlx::query(&postgres_record::drop_unique_index(
            &Self::new_table_name(collection_id),
            index,
        )))
        .await?;
        Ok(())
    }

    async fn postgresdb_insert(&self, db: &PostgresDb) -> Result<()> {
        let mut columns = Vec::with_capacity(self.data.len());
        let mut values = Vec::with_capacity(self.data.len());
        for (col, val) in &self.data {
            columns.push(col.as_str());
            values.push(val);
        }
        let query = postgres_record::insert(&self.table_name, &columns);
        let mut query = sqlx::query(&query);
        for val in values {
            query = val.to_postgresdb_model(query)?;
        }
        db.execute(query).await?;
        Ok(())
    }

    async fn postgresdb_select(
        db: &PostgresDb,
        table_name: &str,
        columns: &Vec<&str>,
        id: &Uuid,
        created_by: &Option<Uuid>,
    ) -> Result<sqlx::postgres::PgRow> {
        Ok(if let Some(created_by) = created_by {
            db.fetch_one_row(
                sqlx::query(&postgres_record::select_by_id_and_created_by(
                    table_name, columns,
                ))
                .bind(id)
                .bind(created_by),
            )
            .await?
        } else {
            db.fetch_one_row(sqlx::query(&postgres_record::select(table_name, columns)).bind(id))
                .await?
        })
    }

    async fn postgresdb_select_many(
        db: &PostgresDb,
        table_name: &str,
        columns: &Vec<&str>,
        created_by: &Option<Uuid>,
        filters: &RecordFilters,
        groups: &Vec<&str>,
        orders: &Vec<RecordOrder>,
        pagination: &RecordPagination,
    ) -> Result<(Vec<sqlx::postgres::PgRow>, i64)> {
        let mut argument_idx = 1;
        let mut filter = filters.postgresdb_filter_query(&None, 0, &mut argument_idx)?;
        if created_by.is_some() {
            if filter.len() > 0 {
                filter = format!("\"_created_by\" = ${argument_idx} AND ({filter})");
            } else {
                filter = format!("\"_created_by\" = ${argument_idx}");
            }
            argument_idx += 1;
        }

        let mut order = Vec::with_capacity(orders.len());
        for o in orders {
            if POSTGRES_ORDER_TYPE.contains(&o.kind.to_uppercase().as_str()) {
                order.push((o.field.as_str(), o.kind.as_str()));
            } else {
                return Err(Error::msg(format!(
                    "Order type '{}' is not supported",
                    &o.kind
                )));
            }
        }

        let query_select_many = postgres_record::select_many(
            table_name,
            &columns,
            &filter,
            groups,
            &order,
            &pagination.limit().is_some(),
            &argument_idx,
        );
        let mut query_select_many = sqlx::query(&query_select_many);
        let query_total = postgres_record::count(table_name, &filter);
        let mut query_total = sqlx::query_as(&query_total);

        if let Some(created_by) = created_by {
            query_select_many = query_select_many.bind(created_by);
            query_total = query_total.bind(created_by);
        }
        query_select_many = filters.postgresdb_values(query_select_many)?;
        if let Some(limit) = pagination.limit() {
            query_select_many = query_select_many.bind(limit);
        }
        query_total = filters.postgresdb_values_as(query_total)?;

        let (rows, total) = tokio::try_join!(
            db.fetch_all_rows(query_select_many),
            db.fetch_one::<(i64,)>(query_total)
        )?;

        Ok((rows, total.0))
    }

    async fn postgresdb_update(&self, db: &PostgresDb) -> Result<()> {
        let mut columns = Vec::with_capacity(self.data.len());
        let mut values = Vec::with_capacity(self.data.len());
        for (col, val) in &self.data {
            if col != "_id" && col != "_updated_at" {
                columns.push(col.as_str());
                values.push(val);
            }
        }
        match self.data.get("_updated_at") {
            Some(updated_at) => {
                columns.push("_updated_at");
                values.push(updated_at);
            }
            None => return Err(Error::msg("_updated_at is undefined")),
        }
        match self.data.get("_id") {
            Some(id) => values.push(id),
            None => return Err(Error::msg("_id is undefined")),
        }
        let query = postgres_record::update(&self.table_name, &columns);
        let mut query = sqlx::query(&query);
        for val in values {
            query = val.to_postgresdb_model(query)?;
        }
        db.execute(query).await?;
        Ok(())
    }

    async fn postgresdb_delete(
        db: &PostgresDb,
        collection_id: &Uuid,
        id: &Uuid,
        created_by: &Option<Uuid>,
    ) -> Result<()> {
        let mut column = HashSet::<String>::with_capacity(2);
        column.insert("_id".to_owned());
        if let Some(created_by) = created_by {
            column.insert("_created_by".to_owned());
            db.execute(
                sqlx::query(&postgres_record::delete(
                    &Self::new_table_name(collection_id),
                    &column,
                ))
                .bind(id)
                .bind(created_by),
            )
            .await?;
        } else {
            db.execute(
                sqlx::query(&postgres_record::delete(
                    &Self::new_table_name(collection_id),
                    &column,
                ))
                .bind(id),
            )
            .await?;
        }
        Ok(())
    }

    async fn postgresdb_delete_expired(
        db: &PostgresDb,
        collection_id: &Uuid,
        ttl_seconds: &i64,
    ) -> Result<()> {
        db.execute(
            sqlx::query(&postgres_record::delete_expired(&Self::new_table_name(
                collection_id,
            )))
            .bind(
                Utc::now()
                    .checked_sub_signed(
                        Duration::try_seconds(*ttl_seconds)
                            .ok_or_else(|| Error::msg("collection ttl is out of range."))?,
                    )
                    .ok_or_else(|| Error::msg("collection ttl is out of range."))?,
            ),
        )
        .await?;
        Ok(())
    }

    async fn mysqldb_create_table(
        db: &MysqlDb,
        collection_id: &Uuid,
        schema_fields: &HashMap<String, SchemaFieldPropsMysqlModel>,
    ) -> Result<()> {
        db.execute_unprepared(sqlx::query(&mysql_record::create_table(
            &Self::new_table_name(collection_id),
            schema_fields,
        )))
        .await?;
        Ok(())
    }

    async fn mysqldb_drop_table(db: &MysqlDb, collection_id: &Uuid) -> Result<()> {
        db.execute_unprepared(sqlx::query(&mysql_record::drop_table(
            &Self::new_table_name(collection_id),
        )))
        .await?;
        Ok(())
    }

    async fn mysqldb_check_table_existence(db: &MysqlDb, collection_id: &Uuid) -> Result<bool> {
        Ok(db
            .fetch_one_unprepared::<(i64,)>(
                sqlx::query_as(MYSQL_COUNT_TABLE).bind(&RecordDao::new_table_name(collection_id)),
            )
            .await?
            .0
            > 0)
    }

    async fn mysqldb_add_columns(
        db: &MysqlDb,
        collection_id: &Uuid,
        columns: &HashMap<String, SchemaFieldPropsMysqlModel>,
    ) -> Result<()> {
        db.execute_unprepared(sqlx::query(&mysql_record::add_columns(
            &Self::new_table_name(collection_id),
            columns,
        )))
        .await?;
        Ok(())
    }

    async fn mysqldb_drop_columns(
        db: &MysqlDb,
        collection_id: &Uuid,
        column_names: &HashSet<String>,
    ) -> Result<()> {
        db.execute_unprepared(sqlx::query(&mysql_record::drop_columns(
            &Self::new_table_name(collection_id),
            column_names,
        )))
        .await?;
        Ok(())
    }

    async fn mysqldb_change_columns_type(
        db: &MysqlDb,
        collection_id: &Uuid,
        columns: &HashMap<String, SchemaFieldPropsMysqlModel>,
    ) -> Result<()> {
        db.execute_unprepared(sqlx::query(&mysql_record::change_columns_type(
            &Self::new_table_name(collection_id),
            columns,
        )))
        .await?;
        Ok(())
    }

    async fn mysqldb_create_index(db: &MysqlDb, collection_id: &Uuid, index: &str) -> Result<()> {
        let record_table = Self::new_table_name(collection_id);

        let does_index_exist =
            db.fetch_one::<(i64,)>(sqlx::query_as(&mysql_record::count_index(
                &record_table,
                index,
            )))
            .await?
            .0 > 0;

        if !does_index_exist {
            db.execute_unprepared(sqlx::query(&mysql_record::create_index(
                &record_table,
                index,
            )))
            .await?;
        }

        Ok(())
    }

    async fn mysqldb_create_unique_index(
        db: &MysqlDb,
        collection_id: &Uuid,
        index: &str,
    ) -> Result<()> {
        let record_table = Self::new_table_name(collection_id);

        let does_index_exist =
            db.fetch_one::<(i64,)>(sqlx::query_as(&mysql_record::count_unique_index(
                &record_table,
                index,
            )))
            .await?
            .0 > 0;

        if !does_index_exist {
            db.execute_unprepared(sqlx::query(&mysql_record::create_unique_index(
                &record_table,
                index,
            )))
            .await?;
        }

        Ok(())
    }

    async fn mysqldb_drop_index(db: &MysqlDb, collection_id: &Uuid, index: &str) -> Result<()> {
        let record_table = Self::new_table_name(collection_id);

        let does_index_exist =
            db.fetch_one::<(i64,)>(sqlx::query_as(&mysql_record::count_index(
                &record_table,
                index,
            )))
            .await?
            .0 > 0;

        if does_index_exist {
            db.execute_unprepared(sqlx::query(&mysql_record::drop_index(
                &Self::new_table_name(collection_id),
                index,
            )))
            .await?;
        }

        Ok(())
    }

    async fn mysqldb_drop_unique_index(
        db: &MysqlDb,
        collection_id: &Uuid,
        index: &str,
    ) -> Result<()> {
        let record_table = Self::new_table_name(collection_id);

        let does_index_exist =
            db.fetch_one::<(i64,)>(sqlx::query_as(&mysql_record::count_unique_index(
                &record_table,
                index,
            )))
            .await?
            .0 > 0;

        if does_index_exist {
            db.execute_unprepared(sqlx::query(&mysql_record::drop_unique_index(
                &Self::new_table_name(collection_id),
                index,
            )))
            .await?;
        }

        Ok(())
    }

    async fn mysqldb_insert(&self, db: &MysqlDb) -> Result<()> {
        let mut columns = Vec::with_capacity(self.data.len());
        let mut values = Vec::with_capacity(self.data.len());
        for (col, val) in &self.data {
            columns.push(col.as_str());
            values.push(val);
        }
        let query = mysql_record::insert(&self.table_name, &columns);
        let mut query = sqlx::query(&query);
        for val in values {
            query = val.to_mysqldb_model(query)?;
        }
        db.execute(query).await?;
        Ok(())
    }

    async fn mysqldb_select(
        db: &MysqlDb,
        table_name: &str,
        columns: &Vec<&str>,
        id: &Uuid,
        created_by: &Option<Uuid>,
    ) -> Result<sqlx::mysql::MySqlRow> {
        Ok(if let Some(created_by) = created_by {
            db.fetch_one_row(
                sqlx::query(&mysql_record::select_by_id_and_created_by(
                    table_name, columns,
                ))
                .bind(id)
                .bind(created_by),
            )
            .await?
        } else {
            db.fetch_one_row(sqlx::query(&mysql_record::select(table_name, columns)).bind(id))
                .await?
        })
    }

    async fn mysqldb_select_many(
        db: &MysqlDb,
        table_name: &str,
        columns: &Vec<&str>,
        created_by: &Option<Uuid>,
        filters: &RecordFilters,
        groups: &Vec<&str>,
        orders: &Vec<RecordOrder>,
        pagination: &RecordPagination,
    ) -> Result<(Vec<sqlx::mysql::MySqlRow>, i64)> {
        let mut filter = filters.mysqldb_filter_query(&None, 0)?;
        if created_by.is_some() {
            if filter.len() > 0 {
                filter = format!("`_created_by` = ? AND ({filter})");
            } else {
                filter = format!("`_created_by` = ?");
            }
        }

        let mut order = Vec::with_capacity(orders.len());
        for o in orders {
            if MYSQL_ORDER_TYPE.contains(&o.kind.to_uppercase().as_str()) {
                order.push((o.field.as_str(), o.kind.as_str()));
            } else {
                return Err(Error::msg(format!(
                    "Order type '{}' is not supported",
                    &o.kind
                )));
            }
        }

        let query_select_many = mysql_record::select_many(
            table_name,
            &columns,
            &filter,
            groups,
            &order,
            &pagination.limit().is_some(),
        );
        let mut query_select_many = sqlx::query(&query_select_many);
        let query_total = mysql_record::count(table_name, &filter);
        let mut query_total = sqlx::query_as(&query_total);

        if let Some(created_by) = created_by {
            query_select_many = query_select_many.bind(created_by);
            query_total = query_total.bind(created_by);
        }
        query_select_many = filters.mysqldb_values(query_select_many)?;
        if let Some(limit) = pagination.limit() {
            query_select_many = query_select_many.bind(limit);
        }
        query_total = filters.mysqldb_values_as(query_total)?;

        let (rows, total) = tokio::try_join!(
            db.fetch_all_rows(query_select_many),
            db.fetch_one::<(i64,)>(query_total)
        )?;

        Ok((rows, total.0))
    }

    async fn mysqldb_update(&self, db: &MysqlDb) -> Result<()> {
        let mut columns = Vec::with_capacity(self.data.len());
        let mut values = Vec::with_capacity(self.data.len());
        for (col, val) in &self.data {
            if col != "_id" && col != "_updated_at" {
                columns.push(col.as_str());
                values.push(val);
            }
        }
        match self.data.get("_updated_at") {
            Some(updated_at) => {
                columns.push("_updated_at");
                values.push(updated_at);
            }
            None => return Err(Error::msg("_updated_at is undefined")),
        }
        match self.data.get("_id") {
            Some(id) => values.push(id),
            None => return Err(Error::msg("_id is undefined")),
        }
        let query = mysql_record::update(&self.table_name, &columns);
        let mut query = sqlx::query(&query);
        for val in values {
            query = val.to_mysqldb_model(query)?;
        }
        db.execute(query).await?;
        Ok(())
    }

    async fn mysqldb_delete(
        db: &MysqlDb,
        collection_id: &Uuid,
        id: &Uuid,
        created_by: &Option<Uuid>,
    ) -> Result<()> {
        let mut column = HashSet::<String>::with_capacity(2);
        column.insert("_id".to_owned());
        if let Some(created_by) = created_by {
            column.insert("_created_by".to_owned());
            db.execute(
                sqlx::query(&mysql_record::delete(
                    &Self::new_table_name(collection_id),
                    &column,
                ))
                .bind(id)
                .bind(created_by),
            )
            .await?;
        } else {
            db.execute(
                sqlx::query(&mysql_record::delete(
                    &Self::new_table_name(collection_id),
                    &column,
                ))
                .bind(id),
            )
            .await?;
        }
        Ok(())
    }

    async fn mysqldb_delete_expired(
        db: &MysqlDb,
        collection_id: &Uuid,
        ttl_seconds: &i64,
    ) -> Result<()> {
        db.execute(
            sqlx::query(&mysql_record::delete_expired(&Self::new_table_name(
                collection_id,
            )))
            .bind(
                Utc::now()
                    .checked_sub_signed(
                        Duration::try_seconds(*ttl_seconds)
                            .ok_or_else(|| Error::msg("collection ttl is out of range."))?,
                    )
                    .ok_or_else(|| Error::msg("collection ttl is out of range."))?,
            ),
        )
        .await?;
        Ok(())
    }

    async fn sqlitedb_create_table(
        db: &SqliteDb,
        collection_id: &Uuid,
        schema_fields: &HashMap<String, SchemaFieldPropsSqliteModel>,
    ) -> Result<()> {
        db.execute_unprepared(sqlx::query(&sqlite_record::create_table(
            &Self::new_table_name(collection_id),
            schema_fields,
        )))
        .await?;
        Ok(())
    }

    async fn sqlite_drop_table(db: &SqliteDb, collection_id: &Uuid) -> Result<()> {
        db.execute_unprepared(sqlx::query(&sqlite_record::drop_table(
            &Self::new_table_name(collection_id),
        )))
        .await?;
        Ok(())
    }

    async fn sqlitedb_check_table_existence(db: &SqliteDb, collection_id: &Uuid) -> Result<bool> {
        Ok(db
            .fetch_one_unprepared::<(i64,)>(
                sqlx::query_as(SQLITE_COUNT_TABLE).bind(&RecordDao::new_table_name(collection_id)),
            )
            .await?
            .0
            > 0)
    }

    async fn sqlitedb_add_columns(
        db: &SqliteDb,
        collection_id: &Uuid,
        columns: &HashMap<String, SchemaFieldPropsSqliteModel>,
    ) -> Result<()> {
        db.execute_unprepared(sqlx::query(&sqlite_record::add_columns(
            &Self::new_table_name(collection_id),
            columns,
        )))
        .await?;
        Ok(())
    }

    async fn sqlitedb_drop_columns(
        db: &SqliteDb,
        collection_id: &Uuid,
        column_names: &HashSet<String>,
    ) -> Result<()> {
        db.execute_unprepared(sqlx::query(&sqlite_record::drop_columns(
            &Self::new_table_name(collection_id),
            column_names,
        )))
        .await?;
        Ok(())
    }

    async fn sqlitedb_change_columns_type(
        db: &SqliteDb,
        collection_id: &Uuid,
        columns: &HashMap<String, SchemaFieldPropsSqliteModel>,
    ) -> Result<()> {
        db.execute_unprepared(sqlx::query(&sqlite_record::change_columns_type(
            &Self::new_table_name(collection_id),
            columns,
        )))
        .await?;
        Ok(())
    }

    async fn sqlitedb_create_index(db: &SqliteDb, collection_id: &Uuid, index: &str) -> Result<()> {
        db.execute_unprepared(sqlx::query(&sqlite_record::create_index(
            &Self::new_table_name(collection_id),
            index,
        )))
        .await?;
        Ok(())
    }

    async fn sqlitedb_create_unique_index(
        db: &SqliteDb,
        collection_id: &Uuid,
        index: &str,
    ) -> Result<()> {
        db.execute_unprepared(sqlx::query(&sqlite_record::create_unique_index(
            &Self::new_table_name(collection_id),
            index,
        )))
        .await?;
        Ok(())
    }

    async fn sqlitedb_drop_index(db: &SqliteDb, collection_id: &Uuid, index: &str) -> Result<()> {
        db.execute_unprepared(sqlx::query(&sqlite_record::drop_index(
            &Self::new_table_name(collection_id),
            index,
        )))
        .await?;
        Ok(())
    }

    async fn sqlitedb_drop_unique_index(
        db: &SqliteDb,
        collection_id: &Uuid,
        index: &str,
    ) -> Result<()> {
        db.execute_unprepared(sqlx::query(&sqlite_record::drop_unique_index(
            &Self::new_table_name(collection_id),
            index,
        )))
        .await?;
        Ok(())
    }

    async fn sqlitedb_insert(&self, db: &SqliteDb) -> Result<()> {
        let mut columns = Vec::with_capacity(self.data.len());
        let mut values = Vec::with_capacity(self.data.len());
        for (col, val) in &self.data {
            columns.push(col.as_str());
            values.push(val);
        }
        let query = sqlite_record::insert(&self.table_name, &columns);
        let mut query = sqlx::query(&query);
        for val in values {
            query = val.to_sqlitedb_model(query)?;
        }
        db.execute(query).await?;
        Ok(())
    }

    async fn sqlitedb_select(
        db: &SqliteDb,
        table_name: &str,
        columns: &Vec<&str>,
        id: &Uuid,
        created_by: &Option<Uuid>,
    ) -> Result<sqlx::sqlite::SqliteRow> {
        Ok(if let Some(created_by) = created_by {
            db.fetch_one_row(
                sqlx::query(&sqlite_record::select_by_id_and_created_by(
                    table_name, columns,
                ))
                .bind(id)
                .bind(created_by),
            )
            .await?
        } else {
            db.fetch_one_row(sqlx::query(&sqlite_record::select(table_name, columns)).bind(id))
                .await?
        })
    }

    async fn sqlitedb_select_many(
        db: &SqliteDb,
        table_name: &str,
        columns: &Vec<&str>,
        created_by: &Option<Uuid>,
        filters: &RecordFilters,
        groups: &Vec<&str>,
        orders: &Vec<RecordOrder>,
        pagination: &RecordPagination,
    ) -> Result<(Vec<sqlx::sqlite::SqliteRow>, i64)> {
        let mut filter = filters.sqlitedb_filter_query(&None, 0)?;
        if created_by.is_some() {
            if filter.len() > 0 {
                filter = format!("\"_created_by\" = ? AND ({filter})");
            } else {
                filter = format!("\"_created_by\" = ?");
            }
        }

        let mut order = Vec::with_capacity(orders.len());
        for o in orders {
            if SQLITE_ORDER_TYPE.contains(&o.kind.to_uppercase().as_str()) {
                order.push((o.field.as_str(), o.kind.as_str()));
            } else {
                return Err(Error::msg(format!(
                    "Order type '{}' is not supported",
                    &o.kind
                )));
            }
        }

        let query_select_many = sqlite_record::select_many(
            table_name,
            &columns,
            &filter,
            groups,
            &order,
            &pagination.limit().is_some(),
        );
        let mut query_select_many = sqlx::query(&query_select_many);
        let query_total = sqlite_record::count(table_name, &filter);
        let mut query_total = sqlx::query_as(&query_total);

        if let Some(created_by) = created_by {
            query_select_many = query_select_many.bind(created_by);
            query_total = query_total.bind(created_by);
        }
        query_select_many = filters.sqlitedb_values(query_select_many)?;
        if let Some(limit) = pagination.limit() {
            query_select_many = query_select_many.bind(limit);
        }
        query_total = filters.sqlitedb_values_as(query_total)?;

        let (rows, total) = tokio::try_join!(
            db.fetch_all_rows(query_select_many),
            db.fetch_one::<(i64,)>(query_total)
        )?;

        Ok((rows, total.0))
    }

    async fn sqlitedb_update(&self, db: &SqliteDb) -> Result<()> {
        let mut columns = Vec::with_capacity(self.data.len());
        let mut values = Vec::with_capacity(self.data.len());
        for (col, val) in &self.data {
            if col != "_id" && col != "_updated_at" {
                columns.push(col.as_str());
                values.push(val);
            }
        }
        match self.data.get("_updated_at") {
            Some(updated_at) => {
                columns.push("_updated_at");
                values.push(updated_at);
            }
            None => return Err(Error::msg("_updated_at is undefined")),
        }
        match self.data.get("_id") {
            Some(id) => values.push(id),
            None => return Err(Error::msg("_id is undefined")),
        }
        let query = sqlite_record::update(&self.table_name, &columns);
        let mut query = sqlx::query(&query);
        for val in values {
            query = val.to_sqlitedb_model(query)?;
        }
        db.execute(query).await?;
        Ok(())
    }

    async fn sqlitedb_delete(
        db: &SqliteDb,
        collection_id: &Uuid,
        id: &Uuid,
        created_by: &Option<Uuid>,
    ) -> Result<()> {
        let mut column = HashSet::<String>::with_capacity(1);
        column.insert("_id".to_owned());
        if let Some(created_by) = created_by {
            column.insert("_created_by".to_owned());
            db.execute(
                sqlx::query(&sqlite_record::delete(
                    &Self::new_table_name(collection_id),
                    &column,
                ))
                .bind(id)
                .bind(created_by),
            )
            .await?;
        } else {
            db.execute(
                sqlx::query(&sqlite_record::delete(
                    &Self::new_table_name(collection_id),
                    &column,
                ))
                .bind(id),
            )
            .await?;
        }
        Ok(())
    }

    async fn sqlitedb_delete_expired(
        db: &SqliteDb,
        collection_id: &Uuid,
        ttl_seconds: &i64,
    ) -> Result<()> {
        db.execute(
            sqlx::query(&sqlite_record::delete_expired(&Self::new_table_name(
                collection_id,
            )))
            .bind(
                Utc::now()
                    .checked_sub_signed(
                        Duration::try_seconds(*ttl_seconds)
                            .ok_or_else(|| Error::msg("collection ttl is out of range."))?,
                    )
                    .ok_or_else(|| Error::msg("collection ttl is out of range."))?,
            ),
        )
        .await?;
        Ok(())
    }
}

#[derive(Clone)]
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
            if idx > 0 {
                if filter.len() > 0 {
                    filter += " ";
                }
                if let Some(operator) = *logical_operator {
                    filter += &format!("{operator} ");
                }
            }
            let op = f.op.to_uppercase();
            if let Some(children) = &f.children {
                if SCYLLA_LOGICAL_OPERATOR.contains(&op.as_str()) {
                    filter += &children.scylladb_filter_query(&Some(&op), level + 1)?;
                } else {
                    return Err(Error::msg(format!(
                        "Operator '{op}' is not supported as a logical operator in ScyllaDB"
                    )));
                }
            } else {
                let field = f.field.as_ref().unwrap();
                if SCYLLA_COMPARISON_OPERATOR.contains(&op.as_str()) {
                    filter += &format!("\"{}\" {}", field, &op);
                    if f.value.is_some() {
                        filter += " ?";
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

    pub fn scylladb_values(&self) -> Result<Vec<Box<dyn SerializeCql + Send + Sync>>> {
        let mut values = Vec::with_capacity(self.values_capacity());
        for f in &self.0 {
            if let Some(value) = &f.value {
                values.push(value.to_scylladb_model()?)
            }
            if let Some(children) = &f.children {
                values.append(&mut children.scylladb_values()?)
            }
        }
        Ok(values)
    }

    pub fn postgresdb_filter_query(
        &self,
        logical_operator: &Option<&str>,
        level: usize,
        first_argument_idx: &mut usize,
    ) -> Result<String> {
        let mut filter = String::new();
        if level > 1 {
            filter += "("
        }
        for (idx, f) in self.0.iter().enumerate() {
            if idx > 0 {
                if filter.len() > 0 {
                    filter += " ";
                }
                if let Some(operator) = *logical_operator {
                    filter += &format!("{operator} ");
                }
            }
            let op = f.op.to_uppercase();
            if let Some(children) = &f.children {
                if POSTGRES_LOGICAL_OPERATOR.contains(&op.as_str()) {
                    filter += &children.postgresdb_filter_query(
                        &Some(&op),
                        level + 1,
                        first_argument_idx,
                    )?;
                } else {
                    return Err(Error::msg(format!(
                        "Operator '{op}' is not supported as a logical operator in PostgreSQL"
                    )));
                }
            } else {
                if POSTGRES_COMPARISON_OPERATOR.contains(&op.as_str()) {
                    filter += &format!("\"{}\" {}", f.field.as_ref().unwrap(), &op);
                    if f.value.is_some() {
                        filter += &format!(" ${}", first_argument_idx);
                        *first_argument_idx += 1;
                    }
                } else {
                    return Err(Error::msg(format!(
                        "Operator '{op}' is not supported as a comparison operator in PostgreSQL"
                    )));
                }
            }
        }
        if level > 1 {
            filter += ")"
        }
        Ok(filter)
    }

    pub fn postgresdb_values<'a>(
        &self,
        query: sqlx::query::Query<'a, sqlx::Postgres, sqlx::postgres::PgArguments>,
    ) -> Result<sqlx::query::Query<'a, sqlx::Postgres, sqlx::postgres::PgArguments>> {
        let mut query = query;
        for f in &self.0 {
            if let Some(value) = &f.value {
                query = value.to_postgresdb_model(query)?
            }
            if let Some(children) = &f.children {
                query = children.postgresdb_values(query)?
            }
        }
        Ok(query)
    }

    pub fn postgresdb_values_as<'a, T>(
        &self,
        query: sqlx::query::QueryAs<'a, sqlx::Postgres, T, sqlx::postgres::PgArguments>,
    ) -> Result<sqlx::query::QueryAs<'a, sqlx::Postgres, T, sqlx::postgres::PgArguments>> {
        let mut query = query;
        for f in &self.0 {
            if let Some(value) = &f.value {
                query = value.to_postgresdb_model_as(query)?
            }
            if let Some(children) = &f.children {
                query = children.postgresdb_values_as(query)?
            }
        }
        Ok(query)
    }

    pub fn mysqldb_filter_query(
        &self,
        logical_operator: &Option<&str>,
        level: usize,
    ) -> Result<String> {
        let mut filter = String::new();
        if level > 1 {
            filter += "("
        }
        for (idx, f) in self.0.iter().enumerate() {
            if idx > 0 {
                if filter.len() > 0 {
                    filter += " ";
                }
                if let Some(operator) = *logical_operator {
                    filter += &format!("{operator} ");
                }
            }
            let op = f.op.to_uppercase();
            if let Some(children) = &f.children {
                if MYSQL_LOGICAL_OPERATOR.contains(&op.as_str()) {
                    filter += &children.mysqldb_filter_query(&Some(&op), level + 1)?;
                } else {
                    return Err(Error::msg(format!(
                        "Operator '{op}' is not supported as a logical operator in MySQL"
                    )));
                }
            } else {
                if MYSQL_COMPARISON_OPERATOR.contains(&op.as_str()) {
                    filter += &format!("`{}` {}", f.field.as_ref().unwrap(), &op);
                    if f.value.is_some() {
                        filter += " ?";
                    }
                } else {
                    return Err(Error::msg(format!(
                        "Operator '{op}' is not supported as a comparison operator in MySQL"
                    )));
                }
            }
        }
        if level > 1 {
            filter += ")"
        }
        Ok(filter)
    }

    pub fn mysqldb_values<'a>(
        &self,
        query: sqlx::query::Query<'a, sqlx::MySql, sqlx::mysql::MySqlArguments>,
    ) -> Result<sqlx::query::Query<'a, sqlx::MySql, sqlx::mysql::MySqlArguments>> {
        let mut query = query;
        for f in &self.0 {
            if let Some(value) = &f.value {
                query = value.to_mysqldb_model(query)?
            }
            if let Some(children) = &f.children {
                query = children.mysqldb_values(query)?
            }
        }
        Ok(query)
    }

    pub fn mysqldb_values_as<'a, T>(
        &self,
        query: sqlx::query::QueryAs<'a, sqlx::MySql, T, sqlx::mysql::MySqlArguments>,
    ) -> Result<sqlx::query::QueryAs<'a, sqlx::MySql, T, sqlx::mysql::MySqlArguments>> {
        let mut query = query;
        for f in &self.0 {
            if let Some(value) = &f.value {
                query = value.to_mysqldb_model_as(query)?
            }
            if let Some(children) = &f.children {
                query = children.mysqldb_values_as(query)?
            }
        }
        Ok(query)
    }

    pub fn sqlitedb_filter_query(
        &self,
        logical_operator: &Option<&str>,
        level: usize,
    ) -> Result<String> {
        let mut filter = String::new();
        if level > 1 {
            filter += "("
        }
        for (idx, f) in self.0.iter().enumerate() {
            if idx > 0 {
                if filter.len() > 0 {
                    filter += " ";
                }
                if let Some(operator) = *logical_operator {
                    filter += &format!("{operator} ");
                }
            }
            let op = f.op.to_uppercase();
            if let Some(children) = &f.children {
                if SQLITE_LOGICAL_OPERATOR.contains(&op.as_str()) {
                    filter += &children.sqlitedb_filter_query(&Some(&op), level + 1)?;
                } else {
                    return Err(Error::msg(format!(
                        "Operator '{op}' is not supported as a logical operator in SQLite"
                    )));
                }
            } else {
                if SQLITE_COMPARISON_OPERATOR.contains(&op.as_str()) {
                    filter += &format!("`{}` {}", f.field.as_ref().unwrap(), &op);
                    if f.value.is_some() {
                        filter += " ?";
                    }
                } else {
                    return Err(Error::msg(format!(
                        "Operator '{op}' is not supported as a comparison operator in SQLite"
                    )));
                }
            }
        }
        if level > 1 {
            filter += ")"
        }
        Ok(filter)
    }

    pub fn sqlitedb_values<'a>(
        &self,
        query: sqlx::query::Query<'a, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'a>>,
    ) -> Result<sqlx::query::Query<'a, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'a>>> {
        let mut query = query;
        for f in &self.0 {
            if let Some(value) = &f.value {
                query = value.to_sqlitedb_model(query)?
            }
            if let Some(children) = &f.children {
                query = children.sqlitedb_values(query)?
            }
        }
        Ok(query)
    }

    pub fn sqlitedb_values_as<'a, T>(
        &self,
        query: sqlx::query::QueryAs<'a, sqlx::Sqlite, T, sqlx::sqlite::SqliteArguments<'a>>,
    ) -> Result<sqlx::query::QueryAs<'a, sqlx::Sqlite, T, sqlx::sqlite::SqliteArguments<'a>>> {
        let mut query = query;
        for f in &self.0 {
            if let Some(value) = &f.value {
                query = value.to_sqlitedb_model_as(query)?
            }
            if let Some(children) = &f.children {
                query = children.sqlitedb_values_as(query)?
            }
        }
        Ok(query)
    }

    fn values_capacity(&self) -> usize {
        let mut capacity = self.0.len();
        for f in &self.0 {
            if let Some(children) = &f.children {
                capacity += children.values_capacity()
            }
        }
        capacity
    }
}

#[derive(Clone)]
pub struct RecordFilter {
    field: Option<String>,
    op: String,
    value: Option<ColumnValue>,
    children: Option<RecordFilters>,
}

impl RecordFilter {
    pub fn new(
        field: &Option<String>,
        op: &str,
        value: &Option<ColumnValue>,
        children: &Option<RecordFilters>,
    ) -> Self {
        Self {
            field: field.to_owned(),
            op: op.to_owned(),
            value: value.clone(),
            children: children.clone(),
        }
    }

    pub fn field(&self) -> &Option<String> {
        &self.field
    }

    pub fn op(&self) -> &str {
        &self.op
    }

    pub fn value(&self) -> &Option<ColumnValue> {
        &self.value
    }

    pub fn children(&self) -> &Option<RecordFilters> {
        &self.children
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
