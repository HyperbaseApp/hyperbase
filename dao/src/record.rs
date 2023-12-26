use std::{collections::hash_map::Keys, str::FromStr};

use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use anyhow::{Error, Result};
use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
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
use num_bigint::BigInt;
use scylla::{
    frame::{
        response::result::CqlValue as ScyllaCqlValue, value::CqlTimestamp as ScyllaCqlTimestamp,
    },
    serialize::value::SerializeCql,
};
use uuid::Uuid;

use crate::{
    collection::{CollectionDao, SchemaFieldKind, SchemaFieldPropsModel},
    util::conversion::{
        self, naivedate_to_scylla_cql_date, naivetime_to_scylla_cql_time,
        scylla_cql_date_to_naivedate, scylla_cql_time_to_naivetime,
    },
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

    pub async fn db_drop_index(db: &Db, collection_id: &Uuid, index: &str) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => Self::scylladb_drop_index(db, collection_id, index).await,
            Db::PostgresqlDb(db) => Self::postgresdb_drop_index(db, collection_id, index).await,
            Db::MysqlDb(db) => Self::mysqldb_drop_index(db, collection_id, index).await,
            Db::SqliteDb(db) => Self::sqlitedb_drop_index(db, collection_id, index).await,
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

    pub async fn db_select(db: &Db, collection_data: &CollectionDao, id: &Uuid) -> Result<Self> {
        match db {
            Db::ScyllaDb(db) => {
                let table_name = Self::new_table_name(collection_data.id());

                let mut columns = Vec::with_capacity(collection_data.schema_fields().len() + 1);
                let mut columns_props =
                    Vec::with_capacity(collection_data.schema_fields().len() + 1);

                columns.push("_id");
                columns_props.push(SchemaFieldPropsModel::new(&SchemaFieldKind::Uuid, &true));

                for (column, props) in collection_data.schema_fields() {
                    columns.push(column);
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
            Db::PostgresqlDb(db) => {
                let table_name = Self::new_table_name(collection_data.id());

                let mut columns = Vec::with_capacity(collection_data.schema_fields().len() + 1);
                columns.push("_id");
                for column in collection_data.schema_fields().keys() {
                    columns.push(column);
                }

                let postgresdb_data =
                    Self::postgresdb_select(db, &table_name, &columns, id).await?;

                let mut data = HashMap::with_capacity(columns.len());
                data.insert(
                    "_id".to_owned(),
                    RecordColumnValue::from_postgresdb_model(
                        &SchemaFieldKind::Uuid,
                        "_id",
                        &postgresdb_data,
                    )?,
                );
                for (field, field_props) in collection_data.schema_fields() {
                    data.insert(
                        field.to_owned(),
                        RecordColumnValue::from_postgresdb_model(
                            field_props.kind(),
                            field,
                            &postgresdb_data,
                        )?,
                    );
                }

                Ok(Self { table_name, data })
            }
            Db::MysqlDb(db) => {
                let table_name = Self::new_table_name(collection_data.id());

                let mut columns = Vec::with_capacity(collection_data.schema_fields().len() + 1);
                columns.push("_id");
                for column in collection_data.schema_fields().keys() {
                    columns.push(column);
                }

                let mysqldb_data = Self::mysqldb_select(db, &table_name, &columns, id).await?;

                let mut data = HashMap::with_capacity(columns.len());
                data.insert(
                    "_id".to_owned(),
                    RecordColumnValue::from_mysqldb_model(
                        &SchemaFieldKind::Uuid,
                        "_id",
                        &mysqldb_data,
                    )?,
                );
                for (field, field_props) in collection_data.schema_fields() {
                    data.insert(
                        field.to_owned(),
                        RecordColumnValue::from_mysqldb_model(
                            field_props.kind(),
                            field,
                            &mysqldb_data,
                        )?,
                    );
                }

                Ok(Self { table_name, data })
            }
            Db::SqliteDb(db) => {
                let table_name = Self::new_table_name(collection_data.id());

                let mut columns = Vec::with_capacity(collection_data.schema_fields().len() + 1);
                columns.push("_id");
                for column in collection_data.schema_fields().keys() {
                    columns.push(column);
                }

                let sqlitedb_data = Self::sqlitedb_select(db, &table_name, &columns, id).await?;

                let mut data = HashMap::with_capacity(columns.len());
                data.insert(
                    "_id".to_owned(),
                    RecordColumnValue::from_sqlitedb_model(
                        &SchemaFieldKind::Uuid,
                        "_id",
                        &sqlitedb_data,
                    )?,
                );
                for (field, field_props) in collection_data.schema_fields() {
                    data.insert(
                        field.to_owned(),
                        RecordColumnValue::from_sqlitedb_model(
                            field_props.kind(),
                            field,
                            &sqlitedb_data,
                        )?,
                    );
                }

                Ok(Self { table_name, data })
            }
        }
    }

    pub async fn db_select_many(
        db: &Db,
        collection_data: &CollectionDao,
        filters: &RecordFilters,
        groups: &Vec<&str>,
        orders: &Vec<RecordOrder>,
        pagination: &RecordPagination,
    ) -> Result<(Vec<Self>, i64)> {
        match db {
            Db::ScyllaDb(db) => {
                let table_name = Self::new_table_name(collection_data.id());

                let mut columns = Vec::with_capacity(collection_data.schema_fields().len() + 1);
                let mut columns_props =
                    Vec::with_capacity(collection_data.schema_fields().len() + 1);

                columns.push("_id");
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
                    groups,
                    orders,
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
                    data_many.push(Self {
                        table_name: table_name.to_owned(),
                        data,
                    });
                }

                Ok((data_many, total))
            }
            Db::PostgresqlDb(db) => {
                let table_name = Self::new_table_name(collection_data.id());

                let mut columns = Vec::with_capacity(collection_data.schema_fields().len() + 1);
                columns.push("_id");
                for column in collection_data.schema_fields().keys() {
                    columns.push(column);
                }

                let (postgres_data_many, total) = Self::postgresdb_select_many(
                    db,
                    &table_name,
                    &columns,
                    filters,
                    groups,
                    orders,
                    pagination,
                )
                .await?;

                let mut data_many = Vec::with_capacity(postgres_data_many.len());
                for postgres_data in &postgres_data_many {
                    let mut data: std::collections::HashMap<_, _, ahash::RandomState> =
                        HashMap::with_capacity(columns.len());
                    data.insert(
                        "_id".to_owned(),
                        RecordColumnValue::from_postgresdb_model(
                            &SchemaFieldKind::Uuid,
                            "_id",
                            postgres_data,
                        )?,
                    );
                    for (field, field_props) in collection_data.schema_fields() {
                        data.insert(
                            field.to_owned(),
                            RecordColumnValue::from_postgresdb_model(
                                field_props.kind(),
                                field,
                                postgres_data,
                            )?,
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

                let mut columns = Vec::with_capacity(collection_data.schema_fields().len() + 1);
                columns.push("_id");
                for column in collection_data.schema_fields().keys() {
                    columns.push(column);
                }

                let (mysql_data_many, total) = Self::mysqldb_select_many(
                    db,
                    &table_name,
                    &columns,
                    filters,
                    groups,
                    orders,
                    pagination,
                )
                .await?;

                let mut data_many = Vec::with_capacity(mysql_data_many.len());
                for mysql_data in &mysql_data_many {
                    let mut data: std::collections::HashMap<_, _, ahash::RandomState> =
                        HashMap::with_capacity(columns.len());
                    data.insert(
                        "_id".to_owned(),
                        RecordColumnValue::from_mysqldb_model(
                            &SchemaFieldKind::Uuid,
                            "_id",
                            mysql_data,
                        )?,
                    );
                    for (field, field_props) in collection_data.schema_fields() {
                        data.insert(
                            field.to_owned(),
                            RecordColumnValue::from_mysqldb_model(
                                field_props.kind(),
                                field,
                                mysql_data,
                            )?,
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

                let mut columns = Vec::with_capacity(collection_data.schema_fields().len() + 1);
                columns.push("_id");
                for column in collection_data.schema_fields().keys() {
                    columns.push(column);
                }

                let (sqlite_data_many, total) = Self::sqlitedb_select_many(
                    db,
                    &table_name,
                    &columns,
                    filters,
                    groups,
                    orders,
                    pagination,
                )
                .await?;

                let mut data_many = Vec::with_capacity(sqlite_data_many.len());
                for sqlite_data in &sqlite_data_many {
                    let mut data: std::collections::HashMap<_, _, ahash::RandomState> =
                        HashMap::with_capacity(columns.len());
                    data.insert(
                        "_id".to_owned(),
                        RecordColumnValue::from_sqlitedb_model(
                            &SchemaFieldKind::Uuid,
                            "_id",
                            sqlite_data,
                        )?,
                    );
                    for (field, field_props) in collection_data.schema_fields() {
                        data.insert(
                            field.to_owned(),
                            RecordColumnValue::from_sqlitedb_model(
                                field_props.kind(),
                                field,
                                sqlite_data,
                            )?,
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

    pub async fn db_update(&self, db: &Db) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => Self::scylladb_update(self, db).await,
            Db::PostgresqlDb(db) => Self::postgresdb_update(self, db).await,
            Db::MysqlDb(db) => Self::mysqldb_update(self, db).await,
            Db::SqliteDb(db) => Self::sqlitedb_update(self, db).await,
        }
    }

    pub async fn db_delete(db: &Db, collection_id: &Uuid, id: &Uuid) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => Self::scylladb_delete(db, collection_id, id).await,
            Db::PostgresqlDb(db) => Self::postgresdb_delete(db, collection_id, id).await,
            Db::MysqlDb(db) => Self::mysqldb_delete(db, collection_id, id).await,
            Db::SqliteDb(db) => Self::sqlitedb_delete(db, collection_id, id).await,
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
        let mut columns: Vec<_> = Vec::with_capacity(self.data.len());
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
    ) -> Result<Vec<Option<ScyllaCqlValue>>> {
        Ok(db
            .execute(&scylla_record::select(table_name, columns), [id].as_ref())
            .await?
            .first_row()?
            .columns)
    }

    async fn scylladb_select_many(
        db: &ScyllaDb,
        table_name: &str,
        columns: &Vec<&str>,
        filters: &RecordFilters,
        groups: &Vec<&str>,
        orders: &Vec<RecordOrder>,
        pagination: &RecordPagination,
    ) -> Result<(Vec<Vec<Option<ScyllaCqlValue>>>, i64)> {
        let filter = filters.scylladb_filter_query(&None, 0)?;

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
        if let Some(limit) = pagination.limit() {
            values.push(Box::new(limit))
        }
        let total_values = filters.scylladb_values()?;

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
            if col != "_id" {
                columns.push(col.as_str());
                values.push(val.to_scylladb_model()?);
            }
        }
        match self.data.get("_id") {
            Some(id) => values.push(id.to_scylladb_model()?),
            None => return Err(Error::msg("Id is undefined")),
        }
        db.execute(&scylla_record::update(&self.table_name, &columns), &values)
            .await?;
        Ok(())
    }

    async fn scylladb_delete(db: &ScyllaDb, collection_id: &Uuid, id: &Uuid) -> Result<()> {
        let mut column = HashSet::<String>::with_capacity(1);
        column.insert("_id".to_owned());
        db.execute(
            &scylla_record::delete(&Self::new_table_name(collection_id), &column),
            [id].as_ref(),
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
    ) -> Result<sqlx::postgres::PgRow> {
        Ok(db
            .fetch_one_row(sqlx::query(&postgres_record::select(table_name, columns)).bind(id))
            .await?)
    }

    async fn postgresdb_select_many(
        db: &PostgresDb,
        table_name: &str,
        columns: &Vec<&str>,
        filters: &RecordFilters,
        groups: &Vec<&str>,
        orders: &Vec<RecordOrder>,
        pagination: &RecordPagination,
    ) -> Result<(Vec<sqlx::postgres::PgRow>, i64)> {
        let mut argument_idx = 1;
        let filter = filters.postgresdb_filter_query(&None, 0, &mut argument_idx)?;

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
            if col != "_id" {
                columns.push(col.as_str());
                values.push(val);
            }
        }
        match self.data.get("_id") {
            Some(id) => values.push(id),
            None => return Err(Error::msg("Id is undefined")),
        }
        let query = postgres_record::update(&self.table_name, &columns);
        let mut query = sqlx::query(&query);
        for val in values {
            query = val.to_postgresdb_model(query)?;
        }
        db.execute(query).await?;
        Ok(())
    }

    async fn postgresdb_delete(db: &PostgresDb, collection_id: &Uuid, id: &Uuid) -> Result<()> {
        let mut column = HashSet::<String>::with_capacity(1);
        column.insert("_id".to_owned());
        db.execute(
            sqlx::query(&postgres_record::delete(
                &Self::new_table_name(collection_id),
                &column,
            ))
            .bind(id),
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
    ) -> Result<sqlx::mysql::MySqlRow> {
        Ok(db
            .fetch_one_row(sqlx::query(&mysql_record::select(table_name, columns)).bind(id))
            .await?)
    }

    async fn mysqldb_select_many(
        db: &MysqlDb,
        table_name: &str,
        columns: &Vec<&str>,
        filters: &RecordFilters,
        groups: &Vec<&str>,
        orders: &Vec<RecordOrder>,
        pagination: &RecordPagination,
    ) -> Result<(Vec<sqlx::mysql::MySqlRow>, i64)> {
        let filter = filters.mysqldb_filter_query(&None, 0)?;

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
            if col != "_id" {
                columns.push(col.as_str());
                values.push(val);
            }
        }
        match self.data.get("_id") {
            Some(id) => values.push(id),
            None => return Err(Error::msg("Id is undefined")),
        }
        let query = mysql_record::update(&self.table_name, &columns);
        let mut query = sqlx::query(&query);
        for val in values {
            query = val.to_mysqldb_model(query)?;
        }
        db.execute(query).await?;
        Ok(())
    }

    async fn mysqldb_delete(db: &MysqlDb, collection_id: &Uuid, id: &Uuid) -> Result<()> {
        let mut column = HashSet::<String>::with_capacity(1);
        column.insert("_id".to_owned());
        db.execute(
            sqlx::query(&mysql_record::delete(
                &Self::new_table_name(collection_id),
                &column,
            ))
            .bind(id),
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

    async fn sqlitedb_drop_index(db: &SqliteDb, collection_id: &Uuid, index: &str) -> Result<()> {
        db.execute_unprepared(sqlx::query(&sqlite_record::drop_index(
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
    ) -> Result<sqlx::sqlite::SqliteRow> {
        Ok(db
            .fetch_one_row(sqlx::query(&sqlite_record::select(table_name, columns)).bind(id))
            .await?)
    }

    async fn sqlitedb_select_many(
        db: &SqliteDb,
        table_name: &str,
        columns: &Vec<&str>,
        filters: &RecordFilters,
        groups: &Vec<&str>,
        orders: &Vec<RecordOrder>,
        pagination: &RecordPagination,
    ) -> Result<(Vec<sqlx::sqlite::SqliteRow>, i64)> {
        let filter = filters.sqlitedb_filter_query(&None, 0)?;

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
            if col != "_id" {
                columns.push(col.as_str());
                values.push(val);
            }
        }
        match self.data.get("_id") {
            Some(id) => values.push(id),
            None => return Err(Error::msg("Id is undefined")),
        }
        let query = sqlite_record::update(&self.table_name, &columns);
        let mut query = sqlx::query(&query);
        for val in values {
            query = val.to_sqlitedb_model(query)?;
        }
        db.execute(query).await?;
        Ok(())
    }

    async fn sqlitedb_delete(db: &SqliteDb, collection_id: &Uuid, id: &Uuid) -> Result<()> {
        let mut column = HashSet::<String>::with_capacity(1);
        column.insert("_id".to_owned());
        db.execute(
            sqlx::query(&sqlite_record::delete(
                &Self::new_table_name(collection_id),
                &column,
            ))
            .bind(id),
        )
        .await?;
        Ok(())
    }
}

#[derive(Clone)]
pub enum RecordColumnValue {
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
    DateTime(Option<DateTime<Utc>>),
    Timestamp(Option<DateTime<Utc>>),
    Json(Option<String>),
}

impl RecordColumnValue {
    pub fn from_serde_json(kind: &SchemaFieldKind, value: &serde_json::Value) -> Result<Self> {
        match value {
            serde_json::Value::Null => Ok(Self::none(kind)),
            serde_json::Value::Bool(value) => match kind {
                SchemaFieldKind::Boolean => Ok(Self::Boolean(Some(*value))),
                SchemaFieldKind::Binary => Ok(Self::Binary(Some(vec![(*value).into()]))),
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
                SchemaFieldKind::Binary => Ok(Self::Binary(Some(value.to_string().into_bytes()))),
                SchemaFieldKind::Json => Ok(Self::Json(Some(value.to_string()))),
                _ => return Err(Error::msg("Wrong value type")),
            },
            serde_json::Value::String(value) => match kind {
                SchemaFieldKind::Varint => Ok(Self::VarInteger(Some(BigInt::from_str(
                    &value.to_string(),
                )?))),
                SchemaFieldKind::Decimal => Ok(Self::Decimal(Some(BigDecimal::from_str(
                    &value.to_string(),
                )?))),
                SchemaFieldKind::String => Ok(Self::String(Some(value.to_owned()))),
                SchemaFieldKind::Binary => Ok(Self::Binary(Some(value.as_bytes().to_vec()))),
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
                    Ok(datetime) => Ok(Self::DateTime(Some(datetime.with_timezone(&Utc)))),
                    Err(err) => Err(err.into()),
                },
                SchemaFieldKind::Timestamp => match DateTime::parse_from_rfc3339(value) {
                    Ok(timestamp) => Ok(Self::Timestamp(Some(timestamp.with_timezone(&Utc)))),
                    Err(err) => Err(err.into()),
                },
                SchemaFieldKind::Json => Ok(Self::Json(Some(value.to_owned()))),
                _ => return Err(Error::msg("Wrong value type")),
            },
            serde_json::Value::Array(value) => match kind {
                SchemaFieldKind::Binary => {
                    let mut bytes = Vec::with_capacity(value.len());
                    for value in value.iter() {
                        match value.as_str() {
                            Some(value) => bytes.append(&mut value.as_bytes().to_vec()),
                            None => return Err(Error::msg("Wrong value type")),
                        }
                    }
                    Ok(Self::Binary(Some(bytes)))
                }
                SchemaFieldKind::Json => Ok(Self::Json(Some(serde_json::json!(value).to_string()))),
                _ => return Err(Error::msg("Wrong value type")),
            },
            serde_json::Value::Object(value) => match kind {
                SchemaFieldKind::Binary => Ok(Self::Binary(Some(
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
            SchemaFieldKind::Boolean => Self::Boolean(None),
            SchemaFieldKind::TinyInt => Self::TinyInteger(None),
            SchemaFieldKind::SmallInt => Self::SmallInteger(None),
            SchemaFieldKind::Int => Self::Integer(None),
            SchemaFieldKind::BigInt => Self::BigInteger(None),
            SchemaFieldKind::Varint => Self::VarInteger(None),
            SchemaFieldKind::Float => Self::Float(None),
            SchemaFieldKind::Double => Self::Double(None),
            SchemaFieldKind::Decimal => Self::Decimal(None),
            SchemaFieldKind::String => Self::String(None),
            SchemaFieldKind::Binary => Self::Binary(None),
            SchemaFieldKind::Uuid => Self::Uuid(None),
            SchemaFieldKind::Date => Self::Date(None),
            SchemaFieldKind::Time => Self::Time(None),
            SchemaFieldKind::DateTime => Self::DateTime(None),
            SchemaFieldKind::Timestamp => Self::Timestamp(None),
            SchemaFieldKind::Json => Self::Json(None),
        }
    }

    pub fn from_scylladb_model(kind: &SchemaFieldKind, value: &ScyllaCqlValue) -> Result<Self> {
        match kind {
            SchemaFieldKind::Boolean => Ok(Self::Boolean(Some(value.as_boolean().ok_or(
                Error::msg(
                    "Incorrect internal value type. Internal value is not of type 'boolean'.",
                ),
            )?))),
            SchemaFieldKind::TinyInt => Ok(Self::TinyInteger(Some(value.as_tinyint().ok_or(
                Error::msg(
                    "Incorrect internal value type. Internal value is not of type 'tinyint'.",
                ),
            )?))),
            SchemaFieldKind::SmallInt => Ok(Self::SmallInteger(Some(value.as_smallint().ok_or(
                Error::msg(
                    "Incorrect internal value type. Internal value is not of type 'smallint'.",
                ),
            )?))),
            SchemaFieldKind::Int => Ok(Self::Integer(Some(value.as_int().ok_or(Error::msg(
                "Incorrect internal value type. Internal value is not of type 'int'.",
            ))?))),
            SchemaFieldKind::BigInt => Ok(Self::BigInteger(Some(value.as_bigint().ok_or(
                Error::msg(
                    "Incorrect internal value type. Internal value is not of type 'bigint'.",
                ),
            )?))),
            SchemaFieldKind::Varint => Ok(Self::VarInteger(Some(BigInt::from_signed_bytes_be(
                &value
                    .clone()
                    .into_varint()
                    .ok_or(Error::msg(
                        "Incorrect internal value type. Internal value is not of type 'varint'.",
                    ))?
                    .to_signed_bytes_be(),
            )))),
            SchemaFieldKind::Float => Ok(Self::Float(Some(value.as_float().ok_or(Error::msg(
                "Incorrect internal value type. Internal value is not of type 'float'.",
            ))?))),
            SchemaFieldKind::Double => {
                Ok(Self::Double(Some(value.as_double().ok_or(Error::msg(
                    "Incorrect internal value type. Internal value is not of type 'double'.",
                ))?)))
            }
            SchemaFieldKind::Decimal => Ok(Self::Decimal(Some(BigDecimal::from_str(
                &value
                    .clone()
                    .into_decimal()
                    .ok_or(Error::msg(
                        "Incorrect internal value type. Internal value is not of type 'decimal'.",
                    ))?
                    .to_string(),
            )?))),
            SchemaFieldKind::String => Ok(Self::String(Some(
                value
                    .as_text()
                    .ok_or(Error::msg(
                        "Incorrect internal value type. Internal value is not of type 'text'.",
                    ))?
                    .to_owned(),
            ))),
            SchemaFieldKind::Binary => Ok(Self::Binary(Some(
                value
                    .as_blob()
                    .ok_or(Error::msg(
                        "Incorrect internal value type. Internal value is not of type 'blob'.",
                    ))?
                    .to_vec(),
            ))),
            SchemaFieldKind::Uuid => Ok(Self::Uuid(Some(value.as_uuid().ok_or(Error::msg(
                "Incorrect internal value type. Internal value is not of type 'uuid'.",
            ))?))),
            SchemaFieldKind::Date => {
                let date = value.as_cql_date().ok_or(Error::msg(
                    "Incorrect internal value type. Internal value is not of type 'date'.",
                ))?;
                Ok(Self::Date(Some(scylla_cql_date_to_naivedate(&date)?)))
            }
            SchemaFieldKind::Time => {
                let time = value.as_cql_time().ok_or(Error::msg(
                    "Incorrect internal value type. Internal value is not of type 'time'.",
                ))?;
                Ok(Self::Time(Some(scylla_cql_time_to_naivetime(&time)?)))
            }
            SchemaFieldKind::DateTime => {
                let timestamp = value.as_cql_timestamp().ok_or(Error::msg(
                    "Incorrect internal value type. Internal value is not of type 'timestamp'.",
                ))?;
                Ok(Self::DateTime(Some(
                    conversion::scylla_cql_timestamp_to_datetime_utc(&timestamp)?,
                )))
            }
            SchemaFieldKind::Timestamp => {
                let timestamp = value.as_cql_timestamp().ok_or(Error::msg(
                    "Incorrect internal value type. Internal value is not of type 'timestamp'.",
                ))?;
                Ok(Self::DateTime(Some(
                    conversion::scylla_cql_timestamp_to_datetime_utc(&timestamp)?,
                )))
            }
            SchemaFieldKind::Json => Ok(Self::Json(Some(String::from_utf8(
                value
                    .as_blob()
                    .ok_or(Error::msg(
                        "Incorrect internal value type. Internal value is not of type 'text'.",
                    ))?
                    .to_owned(),
            )?))),
        }
    }

    pub fn to_scylladb_model(&self) -> Result<Box<dyn SerializeCql>> {
        match self {
            Self::Boolean(data) => Ok(Box::new(*data)),
            Self::TinyInteger(data) => Ok(Box::new(*data)),
            Self::SmallInteger(data) => Ok(Box::new(*data)),
            Self::Integer(data) => Ok(Box::new(*data)),
            Self::BigInteger(data) => Ok(Box::new(*data)),
            Self::VarInteger(data) => Ok(Box::new(match data {
                Some(data) => Some(num_bigint_03::BigInt::from_signed_bytes_be(
                    &data.to_signed_bytes_be(),
                )),
                None => None,
            })),
            Self::Float(data) => Ok(Box::new(*data)),
            Self::Double(data) => Ok(Box::new(*data)),
            Self::Decimal(data) => Ok(Box::new(match data {
                Some(data) => Some(bigdecimal_02::BigDecimal::from_str(&data.to_string())?),
                None => None,
            })),
            Self::String(data) => Ok(Box::new(data.to_owned())),
            Self::Binary(data) => Ok(Box::new(data.to_owned())),
            Self::Uuid(data) => Ok(Box::new(*data)),
            Self::Date(data) => Ok(Box::new(match data {
                Some(data) => Some(naivedate_to_scylla_cql_date(data)?),
                None => None,
            })),
            Self::Time(data) => Ok(Box::new(match data {
                Some(data) => Some(naivetime_to_scylla_cql_time(data)?),
                None => None,
            })),
            Self::DateTime(data) => Ok(Box::new(match data {
                Some(data) => Some(ScyllaCqlTimestamp(data.timestamp_millis())),
                None => None,
            })),
            Self::Timestamp(data) => Ok(Box::new(match data {
                Some(data) => Some(ScyllaCqlTimestamp(data.timestamp_millis())),
                None => None,
            })),
            Self::Json(data) => Ok(Box::new(match data {
                Some(data) => Some(data.to_owned().into_bytes()),
                None => None,
            })),
        }
    }

    pub fn from_postgresdb_model(
        kind: &SchemaFieldKind,
        index: &str,
        value: &sqlx::postgres::PgRow,
    ) -> Result<Self> {
        match kind {
            SchemaFieldKind::Boolean => Ok(Self::Boolean(Some(sqlx::Row::try_get(value, index)?))),
            SchemaFieldKind::TinyInt => {
                Ok(Self::TinyInteger(Some(sqlx::Row::try_get(value, index)?)))
            }
            SchemaFieldKind::SmallInt => {
                Ok(Self::SmallInteger(Some(sqlx::Row::try_get(value, index)?)))
            }
            SchemaFieldKind::Int => Ok(Self::Integer(Some(sqlx::Row::try_get(value, index)?))),
            SchemaFieldKind::BigInt => {
                Ok(Self::BigInteger(Some(sqlx::Row::try_get(value, index)?)))
            }
            SchemaFieldKind::Varint => Ok(Self::VarInteger(Some(BigInt::from_str(
                &sqlx::Row::try_get::<sqlx::types::BigDecimal, _>(value, index)?.to_string(),
            )?))),
            SchemaFieldKind::Float => Ok(Self::Float(Some(sqlx::Row::try_get(value, index)?))),
            SchemaFieldKind::Double => Ok(Self::Double(Some(sqlx::Row::try_get(value, index)?))),
            SchemaFieldKind::Decimal => Ok(Self::Decimal(Some(BigDecimal::from_str(
                &sqlx::Row::try_get::<sqlx::types::BigDecimal, _>(value, index)?.to_string(),
            )?))),
            SchemaFieldKind::String => Ok(Self::String(Some(sqlx::Row::try_get(value, index)?))),
            SchemaFieldKind::Binary => Ok(Self::Binary(Some(sqlx::Row::try_get(value, index)?))),
            SchemaFieldKind::Uuid => Ok(Self::Uuid(Some(sqlx::Row::try_get(value, index)?))),
            SchemaFieldKind::Date => Ok(Self::Date(Some(sqlx::Row::try_get(value, index)?))),
            SchemaFieldKind::Time => Ok(Self::Time(Some(sqlx::Row::try_get(value, index)?))),
            SchemaFieldKind::DateTime => {
                Ok(Self::DateTime(Some(sqlx::Row::try_get(value, index)?)))
            }
            SchemaFieldKind::Timestamp => {
                Ok(Self::Timestamp(Some(sqlx::Row::try_get(value, index)?)))
            }
            SchemaFieldKind::Json => Ok(Self::Json(Some(String::from_utf8(
                (sqlx::Row::try_get::<sqlx::types::Json<Vec<u8>>, _>(value, index)?).0,
            )?))),
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
            Self::DateTime(data) => Ok(query.bind(*data)),
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
            Self::DateTime(data) => Ok(query.bind(*data)),
            Self::Timestamp(data) => Ok(query.bind(*data)),
            Self::Json(data) => Ok(query.bind(match data {
                Some(data) => Some(sqlx::types::Json(data.to_owned().into_bytes())),
                None => None,
            })),
        }
    }

    pub fn from_mysqldb_model(
        kind: &SchemaFieldKind,
        index: &str,
        value: &sqlx::mysql::MySqlRow,
    ) -> Result<Self> {
        match kind {
            SchemaFieldKind::Boolean => Ok(Self::Boolean(Some(sqlx::Row::try_get(value, index)?))),
            SchemaFieldKind::TinyInt => {
                Ok(Self::TinyInteger(Some(sqlx::Row::try_get(value, index)?)))
            }
            SchemaFieldKind::SmallInt => {
                Ok(Self::SmallInteger(Some(sqlx::Row::try_get(value, index)?)))
            }
            SchemaFieldKind::Int => Ok(Self::Integer(Some(sqlx::Row::try_get(value, index)?))),
            SchemaFieldKind::BigInt => {
                Ok(Self::BigInteger(Some(sqlx::Row::try_get(value, index)?)))
            }
            SchemaFieldKind::Varint => Ok(Self::VarInteger(Some(BigInt::from_signed_bytes_be(
                sqlx::Row::try_get::<&[u8], _>(value, index)?,
            )))),
            SchemaFieldKind::Float => Ok(Self::Float(Some(sqlx::Row::try_get(value, index)?))),
            SchemaFieldKind::Double => Ok(Self::Double(Some(sqlx::Row::try_get(value, index)?))),
            SchemaFieldKind::Decimal => Ok(Self::Decimal(Some(BigDecimal::from_str(
                std::str::from_utf8(sqlx::Row::try_get::<&[u8], _>(value, index)?)?,
            )?))),
            SchemaFieldKind::String => Ok(Self::String(Some(sqlx::Row::try_get(value, index)?))),
            SchemaFieldKind::Binary => Ok(Self::Binary(Some(sqlx::Row::try_get(value, index)?))),
            SchemaFieldKind::Uuid => Ok(Self::Uuid(Some(sqlx::Row::try_get(value, index)?))),
            SchemaFieldKind::Date => Ok(Self::Date(Some(sqlx::Row::try_get(value, index)?))),
            SchemaFieldKind::Time => Ok(Self::Time(Some(sqlx::Row::try_get(value, index)?))),
            SchemaFieldKind::DateTime => Ok(Self::DateTime(Some(
                sqlx::Row::try_get::<DateTime<Utc>, _>(value, index)?.into(),
            ))),
            SchemaFieldKind::Timestamp => Ok(Self::DateTime(Some(
                sqlx::Row::try_get::<DateTime<Utc>, _>(value, index)?.into(),
            ))),
            SchemaFieldKind::Json => Ok(Self::Json(Some(String::from_utf8(
                sqlx::Row::try_get::<sqlx::types::Json<Vec<u8>>, _>(value, index)?.0,
            )?))),
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
            Self::DateTime(data) => Ok(query.bind(*data)),
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
            Self::DateTime(data) => Ok(query.bind(*data)),
            Self::Timestamp(data) => Ok(query.bind(*data)),
            Self::Json(data) => Ok(query.bind(match data {
                Some(data) => Some(sqlx::types::Json(data.to_owned().into_bytes())),
                None => None,
            })),
        }
    }

    pub fn from_sqlitedb_model(
        kind: &SchemaFieldKind,
        index: &str,
        value: &sqlx::sqlite::SqliteRow,
    ) -> Result<Self> {
        match kind {
            SchemaFieldKind::Boolean => Ok(Self::Boolean(Some(sqlx::Row::try_get(value, index)?))),
            SchemaFieldKind::TinyInt => {
                Ok(Self::TinyInteger(Some(sqlx::Row::try_get(value, index)?)))
            }
            SchemaFieldKind::SmallInt => {
                Ok(Self::SmallInteger(Some(sqlx::Row::try_get(value, index)?)))
            }
            SchemaFieldKind::Int => Ok(Self::Integer(Some(sqlx::Row::try_get(value, index)?))),
            SchemaFieldKind::BigInt => {
                Ok(Self::BigInteger(Some(sqlx::Row::try_get(value, index)?)))
            }
            SchemaFieldKind::Varint => Ok(Self::VarInteger(Some(BigInt::from_signed_bytes_be(
                sqlx::Row::try_get::<&[u8], _>(value, index)?,
            )))),
            SchemaFieldKind::Float => Ok(Self::Float(Some(sqlx::Row::try_get(value, index)?))),
            SchemaFieldKind::Double => Ok(Self::Double(Some(sqlx::Row::try_get(value, index)?))),
            SchemaFieldKind::Decimal => Ok(Self::Decimal(Some(BigDecimal::from_str(
                std::str::from_utf8(sqlx::Row::try_get::<&[u8], _>(value, index)?)?,
            )?))),
            SchemaFieldKind::String => Ok(Self::String(Some(sqlx::Row::try_get(value, index)?))),
            SchemaFieldKind::Binary => Ok(Self::Binary(Some(sqlx::Row::try_get(value, index)?))),
            SchemaFieldKind::Uuid => Ok(Self::Uuid(Some(sqlx::Row::try_get(value, index)?))),
            SchemaFieldKind::Date => Ok(Self::Date(Some(sqlx::Row::try_get(value, index)?))),
            SchemaFieldKind::Time => Ok(Self::Time(Some(sqlx::Row::try_get(value, index)?))),
            SchemaFieldKind::DateTime => {
                Ok(Self::DateTime(Some(sqlx::Row::try_get(value, index)?)))
            }
            SchemaFieldKind::Timestamp => {
                Ok(Self::Timestamp(Some(sqlx::Row::try_get(value, index)?)))
            }
            SchemaFieldKind::Json => {
                Ok(Self::Json(Some(String::from_utf8(sqlx::Row::try_get::<
                    Vec<u8>,
                    _,
                >(
                    value, index
                )?)?)))
            }
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
            Self::DateTime(data) => Ok(query.bind(*data)),
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
            Self::DateTime(data) => Ok(query.bind(*data)),
            Self::Timestamp(data) => Ok(query.bind(*data)),
            Self::Json(data) => Ok(query.bind(match data {
                Some(data) => Some(data.to_owned().into_bytes()),
                None => None,
            })),
        }
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
            if let Some(child) = &f.child {
                if SCYLLA_LOGICAL_OPERATOR.contains(&op.as_str()) {
                    filter += &child.scylladb_filter_query(&Some(&op), level + 1)?;
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

    pub fn scylladb_values(&self) -> Result<Vec<Box<dyn SerializeCql>>> {
        let mut values = Vec::with_capacity(self.values_capacity());
        for f in &self.0 {
            if let Some(value) = &f.value {
                values.push(value.to_scylladb_model()?)
            }
            if let Some(child) = &f.child {
                values.append(&mut child.scylladb_values()?)
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
            if let Some(child) = &f.child {
                if POSTGRES_LOGICAL_OPERATOR.contains(&op.as_str()) {
                    filter += &child.postgresdb_filter_query(
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
            if let Some(child) = &f.child {
                query = child.postgresdb_values(query)?
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
            if let Some(child) = &f.child {
                query = child.postgresdb_values_as(query)?
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
            if let Some(child) = &f.child {
                if MYSQL_LOGICAL_OPERATOR.contains(&op.as_str()) {
                    filter += &child.mysqldb_filter_query(&Some(&op), level + 1)?;
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
            if let Some(child) = &f.child {
                query = child.mysqldb_values(query)?
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
            if let Some(child) = &f.child {
                query = child.mysqldb_values_as(query)?
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
            if let Some(child) = &f.child {
                if SQLITE_LOGICAL_OPERATOR.contains(&op.as_str()) {
                    filter += &child.sqlitedb_filter_query(&Some(&op), level + 1)?;
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
            if let Some(child) = &f.child {
                query = child.sqlitedb_values(query)?
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
            if let Some(child) = &f.child {
                query = child.sqlitedb_values_as(query)?
            }
        }
        Ok(query)
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

#[derive(Clone)]
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
